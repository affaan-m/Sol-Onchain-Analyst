pub mod api;
pub mod error;

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TimeInterval {
    OneMinute,
    ThreeMinutes,
    FiveMinutes,
    FifteenMinutes,
    OneHour,
    FourHours,
    OneDay,
    OneWeek,
    OneMonth,
}

impl fmt::Display for TimeInterval {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            TimeInterval::OneMinute => "1m",
            TimeInterval::ThreeMinutes => "3m",
            TimeInterval::FiveMinutes => "5m",
            TimeInterval::FifteenMinutes => "15m",
            TimeInterval::OneHour => "1h",
            TimeInterval::FourHours => "4h",
            TimeInterval::OneDay => "1d",
            TimeInterval::OneWeek => "1w",
            TimeInterval::OneMonth => "1M",
        };
        write!(f, "{}", s)
    }
}

impl FromStr for TimeInterval {
    type Err = error::BirdeyeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "1m" => Ok(TimeInterval::OneMinute),
            "3m" => Ok(TimeInterval::ThreeMinutes),
            "5m" => Ok(TimeInterval::FiveMinutes),
            "15m" => Ok(TimeInterval::FifteenMinutes),
            "1h" => Ok(TimeInterval::OneHour),
            "4h" => Ok(TimeInterval::FourHours),
            "1d" => Ok(TimeInterval::OneDay),
            "1w" => Ok(TimeInterval::OneWeek),
            "1M" => Ok(TimeInterval::OneMonth),
            _ => Err(error::BirdeyeError::InvalidTimeInterval(s.to_string())),
        }
    }
}
