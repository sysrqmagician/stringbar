pub mod consts;

use consts::*;

pub struct ByteSizeFormatter {
    divisor: u64,
    unit: String,
}

impl ByteSizeFormatter {
    pub fn new(system: System, magnitude: Magnitude) -> Self {
        let infix = match system {
            System::Binary => "i",
            System::Decimal => "",
        };
        let magnitude = magnitude as usize;
        Self {
            divisor: (system as u64).pow(magnitude as u32),
            unit: format!("{}{infix}B", MAGNITUDE_PREFIXES[magnitude - 1]),
        }
    }

    fn compute_divisor(system: System, magnitude: Magnitude) -> u64 {
        (system as u64).pow(magnitude as u32)
    }

    pub fn fit(value: u64, system: System) -> Self {
        let mut last = Magnitude::Kilo;
        for magnitude in enum_iterator::all::<Magnitude>() {
            if (value as f64 / Self::compute_divisor(system, magnitude) as f64) < 1.0 {
                break;
            }
            last = magnitude;
        }

        Self::new(system, last)
    }

    pub fn get_unit(&self) -> &str {
        &self.unit
    }

    pub fn get_divisor(&self) -> &u64 {
        &self.divisor
    }

    pub fn format(&self, value: u64) -> String {
        format!("{:.2} {}", value as f64 / self.divisor as f64, self.unit)
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn new() {
        let kibibyte = ByteSizeFormatter::new(System::Binary, Magnitude::Kilo);
        assert_eq!("KiB", kibibyte.get_unit());
        assert_eq!(1024_u64, *kibibyte.get_divisor());

        let exabyte = ByteSizeFormatter::new(System::Decimal, Magnitude::Exa);
        assert_eq!("EB", exabyte.get_unit());
        assert_eq!(1_000_000_000_000_000_000_u64, *exabyte.get_divisor());
    }

    #[test]
    fn fit() {
        let kibibyte = ByteSizeFormatter::fit(1, System::Binary);
        assert_eq!("KiB", kibibyte.get_unit());
        assert_eq!(1024_u64, *kibibyte.get_divisor());

        let exabyte = ByteSizeFormatter::fit(1_000_000_000_000_000_001_u64, System::Decimal);
        assert_eq!("EB", exabyte.get_unit());
        assert_eq!(1_000_000_000_000_000_000_u64, *exabyte.get_divisor());
    }

    #[test]
    fn format() {
        let kib = ByteSizeFormatter::new(System::Binary, Magnitude::Kilo);
        assert_eq!("0.50 KiB".to_string(), kib.format(512));
        let gb = ByteSizeFormatter::new(System::Decimal, Magnitude::Giga);
        assert_eq!("1.00 GB".to_string(), gb.format(1_000_000_000));
    }
}
