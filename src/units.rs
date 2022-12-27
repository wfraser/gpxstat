use std::fmt::{Display, Formatter, Result as FmtResult};
use std::str::FromStr;

#[derive(Debug, Copy, Clone)]
pub enum Unit {
    Meters(Meters),
    Kilometers(Kilometers),
    Feet(Feet),
    Miles(Miles),
}

impl Display for Unit {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::Meters(v) => v.fmt(f),
            Self::Kilometers(v) => v.fmt(f),
            Self::Feet(v) => v.fmt(f),
            Self::Miles(v) => v.fmt(f),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
pub struct Meters(pub f64); // todo: use decimal instead

impl FromStr for Meters {
    type Err = std::num::ParseFloatError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(ft) = s.strip_suffix("ft") {
            return Ok(Meters(ft.trim().parse::<f64>()? * 0.3048));
        }
        Ok(Meters(s.parse()?))
    }
}

impl Display for Meters {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{:.1} m", self.0)
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Feet(pub Meters);

impl Display for Feet {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{:.1} ft", (self.0).0 * 3.2808399)
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Miles(pub Meters);

impl Display for Miles {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{:.1} mi", (self.0).0 * 0.00062137119)
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Kilometers(pub Meters);

impl Display for Kilometers {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{:.1} km", (self.0).0 * 0.001)
    }
}
