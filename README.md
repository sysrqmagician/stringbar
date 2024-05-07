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
|MemoryUsage |Memory usage and total      |
|SwapUsage   |Swap usage and total        |
|Timestamp   |A custom formatted timestamp|
|ProcessCount|Number of processes running |
### Example
```ron
(
    separator: " | ",
    update_interval_ms: 500,
    sections: [
        (
            decoration: (
                before: Some("cpu "), // Text to prepend to module output
                after: None, // Text to append to module output
            ),
            module: CpuUsage,
        ),
        (
            decoration: (
                before: Some("dram "),
                after: None,
            ),
            module: MemoryUsage(
                si_units: false, // true for Gigabytes instead of Gibibytes
            ),
        ),
        (
            decoration: (
                before: None,
                after: None,
            ),
            module: Timestamp(
                template: "%A, %d %B %Y | %H:%M:%S", // see https://docs.rs/chrono/latest/chrono/format/strftime/index.html for supported specifiers
            ),
        ),
    ],
)

```
