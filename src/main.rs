use std::{
    fs::OpenOptions,
    io::{BufReader, BufWriter},
    path::Path,
    process::Command,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use bittenhumans::ByteSizeFormatter;
use chrono::Local;
use directories::ProjectDirs;
use notify::{RecommendedWatcher, Watcher};
use ron::{extensions::Extensions, ser::PrettyConfig};
use serde::{Deserialize, Serialize};
use sysinfo::{CpuRefreshKind, Disk, Disks, MemoryRefreshKind, ProcessRefreshKind, System};
use tracing::{error, info};

#[derive(Serialize, Deserialize)]
struct Config {
    separator: String,
    update_interval_ms: u64,
    decimal_data_units: bool,
    sections: Vec<Section>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            separator: " | ".into(),
            update_interval_ms: 1000,
            decimal_data_units: false,
            sections: vec![
                Section {
                    decoration: Decoration {
                        before: Some("dram ".into()),
                        after: None,
                    },
                    module: Module::MemoryUsage,
                },
                Section {
                    decoration: Decoration {
                        before: Some("sda ".into()),
                        after: None,
                    },
                    module: Module::DiskUsage {
                        name: "/dev/sda".into(),
                    },
                },
                Section {
                    decoration: Decoration {
                        before: Some("total ".into()),
                        after: None,
                    },
                    module: Module::DiskUsageTotal {
                        include_removables: false,
                    },
                },
                Section {
                    decoration: Decoration {
                        before: None,
                        after: None,
                    },
                    module: Module::Timestamp {
                        template: "%d/%m/%Y %H:%M".into(),
                    },
                },
            ],
        }
    }
}

#[derive(Serialize, Deserialize)]
enum Module {
    Timestamp { template: String },
    MemoryUsage,
    SwapUsage,
    CpuUsage,
    ProcessCount,
    DiskUsage { name: String },
    DiskUsageTotal { include_removables: bool },
}

#[derive(Serialize, Deserialize)]
struct Decoration {
    before: Option<String>,
    after: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct Section {
    module: Module,
    decoration: Decoration,
}

fn load_config(config_file_path: &Path) -> Option<Config> {
    match OpenOptions::new().read(true).open(config_file_path) {
        Ok(config_file) => match ron::de::from_reader(BufReader::new(config_file)) {
            Ok(x) => Some(x),
            Err(e) => {
                error!("Unable to read config file: {e}");
                None
            }
        },
        Err(e) => match e.kind() {
            std::io::ErrorKind::NotFound => {
                let new_config = Config::default();
                let handle = match OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(config_file_path)
                {
                    Ok(x) => x,
                    Err(e) => {
                        error!("Unable to create new config file: {e}");
                        return None;
                    }
                };

                if let Err(e) = ron::ser::to_writer_pretty(
                    BufWriter::new(handle),
                    &new_config,
                    PrettyConfig::new().extensions(Extensions::all()),
                ) {
                    error!("Unable to write to new config file: {e}");
                    return None;
                }

                info!("Wrote new config file.");
                Some(new_config)
            }
            _ => {
                error!("Unable to open config file: {e}");
                None
            }
        },
    }
}

fn main() {
    tracing_subscriber::fmt().init();

    let dirs = match ProjectDirs::from("", "", "stringbar") {
        Some(x) => x,
        None => {
            error!("Unable to get config directory.");
            return;
        }
    };

    if let Err(e) = std::fs::create_dir_all(dirs.config_dir()) {
        error!("Unable to create config directory: {e}");
        return;
    }

    let config_file_path = dirs.config_dir().join("config.ron");
    let config = Arc::new(Mutex::new(
        load_config(&config_file_path).expect("Initial config load failed, exiting."),
    ));

    let mut watcher;
    {
        let config = config.clone();
        let config_file_path = config_file_path.clone();

        watcher = match RecommendedWatcher::new(
            move |result: Result<notify::Event, notify::Error>| {
                let event = result.unwrap();

                if event.kind.is_modify() {
                    info!("Config file has changed, reloading...");
                    if let Some(new_config) = load_config(&config_file_path) {
                        *config.lock().unwrap() = new_config;
                    }
                }
            },
            notify::Config::default(),
        ) {
            Ok(x) => x,
            Err(e) => {
                error!("Unable to build RecommendedWatcher: {e}");
                return;
            }
        }
    }

    if let Err(e) = watcher.watch(&config_file_path, notify::RecursiveMode::NonRecursive) {
        error!("Unable to start watching config: {e}");
    };

    let mut system = System::new();
    let mut disks = Disks::new();

    loop {
        let mut output = String::new();
        let config = config.lock().unwrap();
        let interval = config.update_interval_ms;
        let mut disks_refreshed = false;

        for section in &config.sections {
            let module_out = match &section.module {
                Module::Timestamp { template } => Local::now().format(template).to_string(),
                Module::MemoryUsage => {
                    system.refresh_memory_specifics(MemoryRefreshKind::new().with_ram());
                    format_byte_usage(
                        system.used_memory(),
                        system.total_memory(),
                        config.decimal_data_units,
                    )
                }
                Module::SwapUsage => {
                    system.refresh_memory_specifics(MemoryRefreshKind::new().with_swap());
                    format_byte_usage(
                        system.used_swap(),
                        system.total_swap(),
                        config.decimal_data_units,
                    )
                }
                Module::CpuUsage => {
                    system.refresh_cpu_specifics(CpuRefreshKind::new().with_cpu_usage());

                    format!("{:.2}%", system.global_cpu_info().cpu_usage())
                }
                Module::ProcessCount => {
                    system.refresh_processes_specifics(ProcessRefreshKind::new());
                    format!("{}", system.processes().len())
                }
                Module::DiskUsage { name } => {
                    if !disks_refreshed {
                        disks.refresh_list();
                        disks_refreshed = true;
                    }

                    if let Some(disk) = disks.iter().find(|x| x.name().to_string_lossy().eq(name)) {
                        let used = disk.total_space() - disk.available_space();

                        format_byte_usage(used, disk.total_space(), config.decimal_data_units)
                    } else {
                        "N/A".into()
                    }
                }
                Module::DiskUsageTotal { include_removables } => {
                    if !disks_refreshed {
                        disks.refresh_list();
                        disks_refreshed = true;
                    }

                    let mut total = 0;
                    let mut used = 0;

                    let mut filtered_disks: Vec<&Disk> = disks.iter().collect();
                    if !include_removables {
                        filtered_disks = disks.iter().filter(|x| !x.is_removable()).collect();
                    }

                    for disk in filtered_disks {
                        total += disk.total_space();
                        used += disk.total_space() - disk.available_space();
                    }

                    format_byte_usage(used, total, config.decimal_data_units)
                }
            };

            if !output.is_empty() {
                output.push_str(&config.separator);
            }

            if let Some(x) = &section.decoration.before {
                output.push_str(x);
            }

            output.push_str(&module_out);

            if let Some(x) = &section.decoration.after {
                output.push_str(x);
            }
        }

        if let Err(e) = Command::new("xsetroot").arg("-name").arg(output).output() {
            error!("Unable to set root window name: {e}");
        }
        drop(config);
        thread::sleep(Duration::from_millis(interval));
    }
}

fn format_byte_usage(used: u64, total: u64, si_units: bool) -> String {
    type System = bittenhumans::consts::System;

    let formatter = ByteSizeFormatter::fit(
        total,
        if si_units {
            System::Decimal
        } else {
            System::Binary
        },
    );

    format!(
        "{}/{}",
        formatter.format(used).split(" ").collect::<Vec<_>>()[0],
        formatter.format(total)
    )
}
