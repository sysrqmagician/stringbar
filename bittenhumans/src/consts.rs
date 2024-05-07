use enum_iterator::Sequence;

pub const MAGNITUDE_PREFIXES: [&str; 6] = ["K", "M", "G", "T", "P", "E"];

#[repr(u8)]
#[derive(Debug, PartialEq, Sequence, Clone, Copy)]
pub enum Magnitude {
    Kilo = 1,
    Mega,
    Giga,
    Tera,
    Peta,
    Exa,
}

#[repr(u16)]
#[derive(Debug, PartialEq, Sequence, Clone, Copy)]
pub enum System {
    Decimal = 1000,
    Binary = 1024,
}
