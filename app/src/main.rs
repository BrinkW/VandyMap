//! VandyMap web app.
//!
//! Phase 0 skeleton. The UI here is deliberately plain: per `CLAUDE.md`, the
//! visual design waits for the maintainer's mockups, so nothing in this file
//! should be read as a design decision.
//!
//! The one non-trivial thing it does is evaluate a sample building's hours
//! against the current instant. That exercises `campus_data`, `chrono`, and
//! `chrono-tz` inside the browser, proving the shared data crate really does
//! work on `wasm32-unknown-unknown` — including chrono-tz's embedded timezone
//! database, which is the part most likely to misbehave there.

use campus_data::hours::{Hours, Time, TimeRange, WeeklySchedule};
use chrono::Utc;
use dioxus::prelude::*;

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    let now = Utc::now();
    let hours = sample_hours();

    let open_state = match hours.is_open_at(now) {
        Some(true) => "open",
        Some(false) => "closed",
        None => "unknown",
    };

    // Formatted outside rsx!: the macro's interpolation takes an identifier or
    // simple expression, not a method call carrying string literals.
    let campus_now = now
        .with_timezone(&campus_data::CAMPUS_TZ)
        .format("%Y-%m-%d %H:%M %Z")
        .to_string();

    rsx! {
        h1 { "VandyMap" }
        p { "Phase 0 skeleton." }
        h2 { "campus_data smoke check" }
        ul {
            li { "Campus time: {campus_now}" }
            li { "Sample building (weekdays 09:00-17:00) is currently: {open_state}" }
        }
    }
}

/// A throwaway schedule used only to prove the data crate runs in the browser.
/// Real building data arrives in Phase 1.
fn sample_hours() -> Hours {
    let nine_to_five = Some(TimeRange {
        open: Time::parse("09:00").expect("literal is a valid HH:MM time"),
        close: Time::parse("17:00").expect("literal is a valid HH:MM time"),
    });

    Hours::Weekly {
        schedule: WeeklySchedule {
            monday: nine_to_five,
            tuesday: nine_to_five,
            wednesday: nine_to_five,
            thursday: nine_to_five,
            friday: nine_to_five,
            saturday: None,
            sunday: None,
        },
    }
}
