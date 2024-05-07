# stringbar
![Crates.io Version](https://img.shields.io/crates/v/stringbar) ![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/sysrqmagician/stringbar/rust.yml) ![GitHub License](https://img.shields.io/github/license/sysrqmagician/stringbar) 

A dwm-style status bar (sets X root window name).

## Features
- Configurable layout
- Hot config reloading

## Runtime Dependencies
- xsetroot

## Installation
### Using cargo
Execute ``cargo install stringbar``
### From source (x86_64 Linux only)
#### Build Dependencies
- [Just](https://github.com/casey/just)
- [UPX](https://github.com/upx/upx)
### Building
Execute ``just release`` and copy the resulting ``./stringbar`` binary to a directory in ``$PATH``.
##  Configuration
- Start stringbar once to generate the default configuration file.
- Edit $XDG_CONFIG_HOME/stringbar/config.ron
### Available modules
|Name        |Description                 |
|------------|----------------------------|
|CpuUsage    |Cpu utilization in percent  |
|MemoryUsage |Memory usage out of total   |
|SwapUsage   |Swap usage out of total     |
|Timestamp   |A custom formatted timestamp|
|ProcessCount|Number of processes running |
|DiskUsage   |Amount of space used out of total on a specific disk|
|DiskUsageTotal|Total amount of space used out of total on all storage devices|
### Example
```ron
#![enable(implicit_some)]
#![enable(unwrap_newtypes)]
#![enable(unwrap_variant_newtypes)]
(
    separator: " | ",
    update_interval_ms: 1000,
    decimal_data_units: false,
    sections: [
        (
            module: MemoryUsage,
            decoration: (
                before: "dram ",
                after: None,
            ),
        ),
        (
            module: DiskUsage(
                name: "/dev/sda",
            ),
            decoration: (
                before: "sda ",
                after: None,
            ),
        ),
        (
            module: DiskUsageTotal(
                include_removables: false,
            ),
            decoration: (
                before: "total ",
                after: None,
            ),
        ),
        (
            module: Timestamp(
                template: "%d/%m/%Y %H:%M",
            ),
            decoration: (
                before: None,
                after: None,
            ),
        ),
    ],
)
```
