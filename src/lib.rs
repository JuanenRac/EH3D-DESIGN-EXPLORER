// SPDX-License-Identifier: GPL-3.0-or-later
//! EH3D-DESIGN-EXPLORER: compare alternatives of a design across several criteria.
//!
//! An alternative is a set of parameters and a set of metrics, and every metric names the tool and the version that produced it. The explorer does not
//! compute the metrics: it compares them. It never reduces the alternatives to one score and never names a single best one: it shows the non-dominated set
//! (the alternatives that no other beats on every criterion), says for every other alternative which one beats it, and shows the difference between two
//! alternatives as a list of parameters and metrics that changed. A parametric sweep is a plain grid in a fixed order, so the same specification always gives
//! the same alternatives.
//!
//! It never applies a change to a design and never creates or starts machine jobs.
#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod cli;
pub mod config;
pub mod error;
pub mod explore;
pub mod sweep;

pub use error::DexpError;

/// The stable code of a finding. Codes are never reused.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Code {
    /// Two alternatives have the same name.
    DuplicateName,
    /// An alternative lacks the metric of a criterion, so it cannot be compared on it.
    MissingMetric,
    /// A metric names no tool and version as its source.
    NoSource,
    /// A number is not finite.
    NotFinite,
}

impl Code {
    /// The text of the code, such as `PHOENIX-DEXP-001`.
    pub fn as_str(self) -> &'static str {
        match self {
            Code::DuplicateName => "PHOENIX-DEXP-001",
            Code::MissingMetric => "PHOENIX-DEXP-002",
            Code::NoSource => "PHOENIX-DEXP-003",
            Code::NotFinite => "PHOENIX-DEXP-004",
        }
    }
}
