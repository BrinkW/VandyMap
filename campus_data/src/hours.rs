//! Building hours, and evaluation of "is it open right now?".
//!
//! Two design decisions here carry real weight:
//!
//! 1. [`Hours`] is an enum, so **"unknown" is representable**. Most buildings
//!    start life with unknown hours during data entry, and an unknown building
//!    must never be silently treated as closed.
//! 2. [`Hours::is_open_at`] takes an absolute instant and converts it to campus
//!    time itself. A viewer in another timezone must still see what is actually
//!    open on campus right now — so the conversion is not left to the caller.

use chrono::{DateTime, Datelike, NaiveTime, TimeZone, Weekday};
use chrono_tz::America::Chicago;
use chrono_tz::Tz;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Vanderbilt's campus timezone. All "open now" evaluation happens here,
/// regardless of where the viewer is.
pub const CAMPUS_TZ: Tz = Chicago;

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum Hours {
    /// No data yet. Excluded from "open now" results — never assumed open or closed.
    #[default]
    Unknown,
    /// 24/7, or effectively always accessible.
    AlwaysOpen,
    Weekly {
        schedule: WeeklySchedule,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct WeeklySchedule {
    /// `None` means closed that day.
    #[serde(default)]
    pub monday: Option<TimeRange>,
    #[serde(default)]
    pub tuesday: Option<TimeRange>,
    #[serde(default)]
    pub wednesday: Option<TimeRange>,
    #[serde(default)]
    pub thursday: Option<TimeRange>,
    #[serde(default)]
    pub friday: Option<TimeRange>,
    #[serde(default)]
    pub saturday: Option<TimeRange>,
    #[serde(default)]
    pub sunday: Option<TimeRange>,
}

impl WeeklySchedule {
    pub fn for_weekday(&self, day: Weekday) -> Option<&TimeRange> {
        match day {
            Weekday::Mon => self.monday.as_ref(),
            Weekday::Tue => self.tuesday.as_ref(),
            Weekday::Wed => self.wednesday.as_ref(),
            Weekday::Thu => self.thursday.as_ref(),
            Weekday::Fri => self.friday.as_ref(),
            Weekday::Sat => self.saturday.as_ref(),
            Weekday::Sun => self.sunday.as_ref(),
        }
    }

    /// Every day that has a range, for validation and display.
    pub fn all_ranges(&self) -> impl Iterator<Item = (Weekday, &TimeRange)> {
        [
            (Weekday::Mon, self.monday.as_ref()),
            (Weekday::Tue, self.tuesday.as_ref()),
            (Weekday::Wed, self.wednesday.as_ref()),
            (Weekday::Thu, self.thursday.as_ref()),
            (Weekday::Fri, self.friday.as_ref()),
            (Weekday::Sat, self.saturday.as_ref()),
            (Weekday::Sun, self.sunday.as_ref()),
        ]
        .into_iter()
        .filter_map(|(day, range)| range.map(|r| (day, r)))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimeRange {
    pub open: Time,
    pub close: Time,
}

/// A wall-clock time of day, serialized as `"HH:MM"`.
///
/// Kept as a parsed type rather than a `String` so that `"25:00"` fails at load
/// time instead of silently becoming bad data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Time(pub NaiveTime);

impl Time {
    pub const FORMAT: &'static str = "%H:%M";

    pub fn parse(raw: &str) -> Result<Self, chrono::ParseError> {
        NaiveTime::parse_from_str(raw, Self::FORMAT).map(Time)
    }
}

impl std::fmt::Display for Time {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.format(Self::FORMAT))
    }
}

impl Serialize for Time {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for Time {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let raw = String::deserialize(deserializer)?;
        Time::parse(&raw).map_err(|_| {
            serde::de::Error::custom(format!("expected a time as \"HH:MM\", got {raw:?}"))
        })
    }
}

impl Hours {
    /// Whether the building is open at `when`.
    ///
    /// Returns `None` when hours are unknown — the caller must decide how to
    /// present that, and per `docs/SPEC.md` unknown buildings are excluded from
    /// "open now" results rather than assumed open or closed.
    ///
    /// `when` is an absolute instant in any timezone; it is converted to
    /// [`CAMPUS_TZ`] before evaluation, so a viewer browsing from another
    /// timezone still sees what is open on campus.
    pub fn is_open_at<T: TimeZone>(&self, when: DateTime<T>) -> Option<bool> {
        match self {
            Hours::Unknown => None,
            Hours::AlwaysOpen => Some(true),
            Hours::Weekly { schedule } => {
                let campus_time = when.with_timezone(&CAMPUS_TZ);
                let Some(range) = schedule.for_weekday(campus_time.weekday()) else {
                    return Some(false);
                };
                let now = Time(campus_time.time());
                // Half-open: a building closing at 22:00 is shut at 22:00.
                Some(now >= range.open && now < range.close)
            }
        }
    }

    /// Whether these hours can participate in "open now" filtering at all.
    pub fn is_known(&self) -> bool {
        !matches!(self, Hours::Unknown)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn range(open: &str, close: &str) -> Option<TimeRange> {
        Some(TimeRange {
            open: Time::parse(open).unwrap(),
            close: Time::parse(close).unwrap(),
        })
    }

    fn weekdays_9_to_5() -> Hours {
        Hours::Weekly {
            schedule: WeeklySchedule {
                monday: range("09:00", "17:00"),
                tuesday: range("09:00", "17:00"),
                wednesday: range("09:00", "17:00"),
                thursday: range("09:00", "17:00"),
                friday: range("09:00", "17:00"),
                saturday: None,
                sunday: None,
            },
        }
    }

    /// 2026-09-09 is a Wednesday.
    fn campus_time(hour: u32, minute: u32) -> DateTime<Tz> {
        CAMPUS_TZ
            .with_ymd_and_hms(2026, 9, 9, hour, minute, 0)
            .unwrap()
    }

    #[test]
    fn unknown_hours_are_neither_open_nor_closed() {
        assert_eq!(Hours::Unknown.is_open_at(campus_time(12, 0)), None);
        assert!(!Hours::Unknown.is_known());
    }

    #[test]
    fn always_open_is_open_at_any_hour() {
        assert_eq!(Hours::AlwaysOpen.is_open_at(campus_time(3, 30)), Some(true));
    }

    #[test]
    fn weekly_hours_respect_the_open_window() {
        let hours = weekdays_9_to_5();
        assert_eq!(hours.is_open_at(campus_time(8, 59)), Some(false));
        assert_eq!(hours.is_open_at(campus_time(9, 0)), Some(true));
        assert_eq!(hours.is_open_at(campus_time(16, 59)), Some(true));
        // Closing time itself is closed, not open.
        assert_eq!(hours.is_open_at(campus_time(17, 0)), Some(false));
    }

    #[test]
    fn a_day_with_no_range_is_closed() {
        // 2026-09-12 is a Saturday.
        let saturday = CAMPUS_TZ.with_ymd_and_hms(2026, 9, 12, 12, 0, 0).unwrap();
        assert_eq!(weekdays_9_to_5().is_open_at(saturday), Some(false));
    }

    /// The bug this guards against: evaluating "open now" in the *viewer's*
    /// timezone rather than the campus's. A viewer in Los Angeles at 07:30
    /// local is looking at 09:30 on campus, so the building is open.
    #[test]
    fn evaluation_uses_campus_time_not_viewer_time() {
        let hours = weekdays_9_to_5();

        let la = chrono_tz::America::Los_Angeles
            .with_ymd_and_hms(2026, 9, 9, 7, 30, 0)
            .unwrap();
        assert_eq!(
            hours.is_open_at(la),
            Some(true),
            "07:30 in Los Angeles is 09:30 on campus, which is open"
        );

        // And the reverse: 16:30 in LA is 18:30 on campus, which is shut.
        let la_evening = chrono_tz::America::Los_Angeles
            .with_ymd_and_hms(2026, 9, 9, 16, 30, 0)
            .unwrap();
        assert_eq!(hours.is_open_at(la_evening), Some(false));
    }

    #[test]
    fn utc_instants_are_converted_to_campus_time() {
        // 14:30 UTC on 2026-09-09 is 09:30 CDT.
        let utc = Utc.with_ymd_and_hms(2026, 9, 9, 14, 30, 0).unwrap();
        assert_eq!(weekdays_9_to_5().is_open_at(utc), Some(true));
    }

    #[test]
    fn times_round_trip_and_reject_nonsense() {
        assert_eq!(Time::parse("07:00").unwrap().to_string(), "07:00");
        assert!(Time::parse("25:00").is_err());
        assert!(Time::parse("noon").is_err());
    }
}
