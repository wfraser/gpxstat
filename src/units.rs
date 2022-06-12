use std::fmt::{Display, Formatter, Result as FmtResult};
use std::str::FromStr;

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
pub struct Meters(pub f64); // todo: use decimal instead

impl FromStr for Meters {
    type Err = std::num::ParseFloatError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
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
