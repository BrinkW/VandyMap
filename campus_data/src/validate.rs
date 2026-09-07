//! Schema and consistency checks over `data/buildings/`.
//!
//! Run in CI by `cargo run -p xtask -- validate-data`. On a solo project with no
//! second reviewer, this is the only thing standing between a typo and shipped
//! bad data, so the rules are deliberately strict and the messages specific.

use std::collections::HashMap;

use chrono::NaiveDate;

use crate::building::{Building, LatLon};
use crate::hours::Hours;

/// Generous bounding box around the in-scope campus (see `docs/SPEC.md`).
///
/// Wide enough to cover the main core, Peabody, Highland Quad, the athletics
/// facilities, and the Data Science Institute, but tight enough that a
/// transposed coordinate lands far outside it.
pub const CAMPUS_BOUNDS: Bounds = Bounds {
    min_lat: 36.11,
    max_lat: 36.18,
    min_lon: -86.84,
    max_lon: -86.77,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Bounds {
    pub min_lat: f64,
    pub max_lat: f64,
    pub min_lon: f64,
    pub max_lon: f64,
}

impl Bounds {
    pub fn contains(&self, point: LatLon) -> bool {
        (self.min_lat..=self.max_lat).contains(&point.lat)
            && (self.min_lon..=self.max_lon).contains(&point.lon)
    }
}

/// One problem found in one building file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    /// File stem the problem was found in, e.g. `"featheringill-hall"`.
    pub file: String,
    pub message: String,
}

impl std::fmt::Display for Finding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.toml: {}", self.file, self.message)
    }
}

/// Check one building. `file_stem` is the filename without its extension.
///
/// `today` is passed in rather than read from the clock so that results are
/// deterministic and testable.
pub fn validate_building(building: &Building, file_stem: &str, today: NaiveDate) -> Vec<Finding> {
    let mut findings = Vec::new();
    let mut report = |message: String| {
        findings.push(Finding {
            file: file_stem.to_string(),
            message,
        })
    };

    if building.id != file_stem {
        report(format!(
            "id {:?} does not match the filename {:?}",
            building.id, file_stem
        ));
    }

    if building.name.trim().is_empty() {
        report("name is empty".to_string());
    }

    if building.categories.is_empty() {
        report("categories is empty; every building needs at least one".to_string());
    }

    if !CAMPUS_BOUNDS.contains(building.location.centroid) {
        report(format!(
            "centroid {:?} is outside the campus bounds — check for transposed lat/lon",
            building.location.centroid
        ));
    }

    let footprint = &building.location.footprint;
    if !footprint.is_empty() && footprint.len() < 3 {
        report(format!(
            "footprint has {} points; a polygon needs at least 3",
            footprint.len()
        ));
    }
    for (index, point) in footprint.iter().enumerate() {
        if !CAMPUS_BOUNDS.contains(*point) {
            report(format!(
                "footprint point {index} {point:?} is outside the campus bounds"
            ));
        }
    }

    if let Hours::Weekly { schedule } = &building.hours {
        for (day, range) in schedule.all_ranges() {
            if range.close <= range.open {
                report(format!(
                    "{day:?} closes at {} which is not after its {} opening \
                     (past-midnight closing times are not representable in v1)",
                    range.close, range.open
                ));
            }
        }
    }

    if let Some(verified) = building.last_verified {
        if verified > today {
            report(format!("last_verified {verified} is in the future"));
        }
    }

    findings
}

/// Check a whole set of buildings, including cross-file rules.
///
/// Takes `(file_stem, building)` pairs.
pub fn validate_all(buildings: &[(String, Building)], today: NaiveDate) -> Vec<Finding> {
    let mut findings = Vec::new();

    for (file_stem, building) in buildings {
        findings.extend(validate_building(building, file_stem, today));
    }

    let mut seen: HashMap<&str, &str> = HashMap::new();
    for (file_stem, building) in buildings {
        if let Some(previous) = seen.insert(&building.id, file_stem) {
            findings.push(Finding {
                file: file_stem.clone(),
                message: format!("id {:?} is already used by {previous}.toml", building.id),
            });
        }
    }

    findings
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::building::{Category, Location};
    use crate::hours::{Time, TimeRange, WeeklySchedule};

    fn today() -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 9, 7).unwrap()
    }

    fn valid_building() -> Building {
        Building {
            id: "kirkland-hall".to_string(),
            name: "Kirkland Hall".to_string(),
            aliases: vec![],
            categories: vec![Category::Administrative],
            address: "2201 West End Ave, Nashville, TN 37235".to_string(),
            description: String::new(),
            hours: Hours::Unknown,
            amenities: vec![],
            departments: vec![],
            external_link: None,
            photos: vec![],
            location: Location {
                centroid: LatLon::new(36.1463, -86.8033),
                footprint: vec![],
                source: None,
            },
            last_verified: None,
            floors: vec![],
        }
    }

    #[test]
    fn a_good_building_produces_no_findings() {
        assert_eq!(
            validate_building(&valid_building(), "kirkland-hall", today()),
            vec![]
        );
    }

    #[test]
    fn id_must_match_the_filename() {
        let findings = validate_building(&valid_building(), "kirkland_hall", today());
        assert!(findings.iter().any(|f| f.message.contains("filename")));
    }

    #[test]
    fn categories_cannot_be_empty() {
        let mut building = valid_building();
        building.categories.clear();
        let findings = validate_building(&building, "kirkland-hall", today());
        assert!(findings.iter().any(|f| f.message.contains("categories")));
    }

    /// The case the LatLon newtype and this bounds check exist for.
    #[test]
    fn transposed_coordinates_are_caught() {
        let mut building = valid_building();
        building.location.centroid = LatLon::new(-86.8033, 36.1463);
        let findings = validate_building(&building, "kirkland-hall", today());
        assert!(findings.iter().any(|f| f.message.contains("transposed")));
    }

    #[test]
    fn a_degenerate_footprint_is_rejected() {
        let mut building = valid_building();
        building.location.footprint =
            vec![LatLon::new(36.146, -86.803), LatLon::new(36.147, -86.804)];
        let findings = validate_building(&building, "kirkland-hall", today());
        assert!(findings.iter().any(|f| f.message.contains("at least 3")));
    }

    #[test]
    fn closing_before_opening_is_rejected() {
        let mut building = valid_building();
        building.hours = Hours::Weekly {
            schedule: WeeklySchedule {
                monday: Some(TimeRange {
                    open: Time::parse("17:00").unwrap(),
                    close: Time::parse("09:00").unwrap(),
                }),
                ..Default::default()
            },
        };
        let findings = validate_building(&building, "kirkland-hall", today());
        assert!(findings.iter().any(|f| f.message.contains("not after")));
    }

    #[test]
    fn last_verified_cannot_be_in_the_future() {
        let mut building = valid_building();
        building.last_verified = NaiveDate::from_ymd_opt(2027, 1, 1);
        let findings = validate_building(&building, "kirkland-hall", today());
        assert!(findings.iter().any(|f| f.message.contains("future")));
    }

    #[test]
    fn duplicate_ids_across_files_are_caught() {
        let building = valid_building();
        let buildings = vec![
            ("kirkland-hall".to_string(), building.clone()),
            ("kirkland-hall-2".to_string(), building),
        ];
        let findings = validate_all(&buildings, today());
        assert!(findings.iter().any(|f| f.message.contains("already used")));
    }

    #[test]
    fn an_empty_dataset_is_valid() {
        assert_eq!(validate_all(&[], today()), vec![]);
    }
}
