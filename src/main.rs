use std::{
    fs::OpenOptions,
    io::{BufReader, BufWriter},
    path::Path,
    process::Command,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use chrono::Local;
use directories::ProjectDirs;
use notify::{RecommendedWatcher, Watcher};
use ron::ser::PrettyConfig;
use serde::{Deserialize, Serialize};
use sysinfo::{CpuRefreshKind, MemoryRefreshKind, System};
use tracing::{error, info};

#[derive(Serialize, Deserialize)]
struct Config {
    separator: String,
    update_interval_ms: u64,
    sections: Vec<Section>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            separator: " | ".into(),
            update_interval_ms: 1000,
            sections: vec![
                Section {
                    decoration: Decoration {
                        before: Some("dram ".into()),
                        after: None,
                    },
                    module: Module::MemoryUsage { si_units: false },
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
    MemoryUsage { si_units: bool },
    SwapUsage { si_units: bool },
    CpuUsage,
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
                    PrettyConfig::new(),
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
    let mut system = System::new();
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

    loop {
        let mut output = String::new();
        let config = config.lock().unwrap();
        let interval = config.update_interval_ms;
        for section in &config.sections {
            let module_out = match &section.module {
                Module::Timestamp { template } => Local::now().format(template).to_string(),
                Module::MemoryUsage { si_units } => {
                    system.refresh_memory_specifics(MemoryRefreshKind::new().with_ram());
                    format!(
                        "{:.2}/{:.2}G",
                        format_memory(system.used_memory(), *si_units),
                        format_memory(system.total_memory(), *si_units)
                    )
                }
                Module::SwapUsage { si_units } => {
                    system.refresh_memory_specifics(MemoryRefreshKind::new().with_swap());
                    format!(
                        "{:.2}/{:.2}G",
                        format_memory(system.used_swap(), *si_units),
                        format_memory(system.total_swap(), *si_units)
                    )
                }
                Module::CpuUsage => {
                    system.refresh_cpu_specifics(CpuRefreshKind::new().with_cpu_usage());

                    format!("{:.2}%", system.global_cpu_info().cpu_usage())
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

fn format_memory(value: u64, si_units: bool) -> f32 {
    if si_units {
        value as f32 / 1000000000.0
    } else {
        value as f32 / 1073741824.0
    }
}
