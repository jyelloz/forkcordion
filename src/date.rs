use std::fmt;
use derive_more::{From, Into};
use time::OffsetDateTime;

use deku::prelude::*;

/// UNIX timestamp for 2000-01-01T00:00:00Z
pub const MAC_EPOCH: i64 = 9_4668_4800;

/// Mac file timestamp: the number of seconds before or after
/// the start of the year 2000.
#[derive(Default, DekuRead, DekuWrite, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, From, Into)]
pub struct Date(#[deku(endian = "big")] i32);

impl Date {
    fn as_unix_timestamp(&self) -> i64 {
        self.0 as i64 + MAC_EPOCH
    }
}

impl TryInto<OffsetDateTime> for &Date {
    type Error = time::error::ComponentRange;
    fn try_into(self) -> Result<OffsetDateTime, Self::Error> {
        OffsetDateTime::from_unix_timestamp(self.as_unix_timestamp())
    }
}

impl fmt::Display for Date {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let date: Option<OffsetDateTime> = self.try_into().ok();
        if let Some(date) = date {
            write!(f, "{}", date)
        } else {
            write!(f, "{}", self.0)
        }
    }
}

impl fmt::Debug for Date {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Date({})", self)
    }
}

/// All the dates that the Finder will record for a file.
#[derive(Debug, DekuRead, DekuWrite, Default, Clone, Copy, PartialEq, Eq)]
pub struct Dates {
    pub create: Date,
    pub modify: Date,
    pub backup: Date,
    pub access: Date,
}
