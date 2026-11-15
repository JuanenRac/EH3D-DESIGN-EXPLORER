// SPDX-License-Identifier: GPL-3.0-or-later
//! The simulation mode: a host twin of the tools that compute the figures of an alternative, with faults and the recovery from them.
//!
//! The figures of a study come from other tools of the suite, which do not run here. [`SolverTwin`] stands in for them: it computes the figures of an
//! alternative from its parameters with a fixed formula plus a seeded deviation (it never reads the clock), so the same seed always gives the same study.
//! [`Failure`] makes the twin behave as a faulty producer would, and [`recover`] runs the failed alternatives again. Nothing here has run against a real tool.

use crate::config::Config;
use crate::explore::{Candidate, Goal, Metric, Study};
use crate::sweep::{expand, SweepSpec};
use std::collections::BTreeMap;

/// A small deterministic generator (SplitMix64): the same seed always gives the same numbers.
#[derive(Debug, Clone)]
pub struct SimRng(u64);

impl SimRng {
    /// A generator that starts from `seed`.
    pub fn new(seed: u64) -> SimRng {
        SimRng(seed)
    }

    /// The next number.
    pub fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// A number in `[0, 1)`.
    pub fn unit(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }
}

/// How the twin of a figure-producing tool can fail.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Failure {
    /// The tool crashed before writing the figure of this criterion.
    Crash,
    /// The tool wrote a figure but not its name and version.
    NoSource,
    /// The tool diverged and wrote a number that is not finite.
    Diverged,
}

/// The twin of the tools that compute the figures: cost goes up with the size, the losses go down with it.
#[derive(Debug, Clone)]
pub struct SolverTwin {
    rng: SimRng,
    source: String,
}

/// The criteria of the twin.
pub fn criteria() -> BTreeMap<String, Goal> {
    let mut c = BTreeMap::new();
    c.insert("cost_eur".to_string(), Goal::Min);
    c.insert("loss_w".to_string(), Goal::Min);
    c.insert("margin_pct".to_string(), Goal::Max);
    c
}

impl SolverTwin {
    /// A twin that gives the same figures for the same seed.
    pub fn new(seed: u64) -> SolverTwin {
        SolverTwin {
            rng: SimRng::new(seed),
            source: "solver-twin 0.0.0".to_string(),
        }
    }

    /// The figures of an alternative with these parameters (`size_mm` and `gap_mm`).
    pub fn evaluate(&mut self, name: &str, parameters: &BTreeMap<String, f64>) -> Candidate {
        let size = parameters.get("size_mm").copied().unwrap_or(1.0);
        let gap = parameters.get("gap_mm").copied().unwrap_or(1.0);
        let noise = self.rng.unit();
        let figures = [
            ("cost_eur", 2.0 * size + 2.0 * gap + noise),
            ("loss_w", 40.0 / size + gap + noise),
            ("margin_pct", 10.0 * size - 2.0 * gap + noise),
        ];
        let metrics = figures
            .iter()
            .map(|(k, v)| {
                (
                    (*k).to_string(),
                    Metric {
                        value: *v,
                        source: self.source.clone(),
                    },
                )
            })
            .collect();
        Candidate {
            name: name.to_string(),
            parameters: parameters.clone(),
            metrics,
        }
    }

    /// A study made of the grid of a sweep.
    pub fn study(&mut self, spec: &SweepSpec, config: &Config) -> Study {
        let grid = expand(spec, config).expect("the specification of the twin is valid");
        let candidates = grid
            .iter()
            .map(|g| self.evaluate(&g.name, &g.parameters))
            .collect();
        Study {
            criteria: criteria(),
            candidates,
        }
    }
}

/// Make the figure of one criterion of one alternative fail. Returns `false` when there is no such alternative or criterion.
pub fn inject(study: &mut Study, name: &str, criterion: &str, failure: Failure) -> bool {
    let Some(candidate) = study.candidates.iter_mut().find(|c| c.name == name) else {
        return false;
    };
    match failure {
        Failure::Crash => candidate.metrics.remove(criterion).is_some(),
        Failure::NoSource => match candidate.metrics.get_mut(criterion) {
            Some(m) => {
                m.source.clear();
                true
            }
            None => false,
        },
        Failure::Diverged => match candidate.metrics.get_mut(criterion) {
            Some(m) => {
                m.value = f64::NAN;
                true
            }
            None => false,
        },
    }
}

/// The recovery: run again, with the twin, every alternative that the check of the study reports, and say how many were run again.
pub fn recover(study: &mut Study, twin: &mut SolverTwin) -> usize {
    let mut again: Vec<String> = study.check().into_iter().map(|f| f.subject).collect();
    again.sort();
    again.dedup();
    for name in &again {
        if let Some(at) = study.candidates.iter().position(|c| &c.name == name) {
            let parameters = study.candidates[at].parameters.clone();
            study.candidates[at] = twin.evaluate(name, &parameters);
        }
    }
    again.len()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Code;

    fn spec() -> SweepSpec {
        serde_json::from_str(
            r#"{"prefix":"d","parameters":{"size_mm":[1,2,3,4,5],"gap_mm":[0.5,1,2]}}"#,
        )
        .unwrap()
    }

    fn study(seed: u64) -> Study {
        SolverTwin::new(seed).study(&spec(), &Config::default())
    }

    #[test]
    fn the_same_seed_gives_the_same_study_and_another_seed_other_figures() {
        let (a, b, c) = (study(4), study(4), study(5));
        assert_eq!(a, b);
        assert_ne!(a, c);
        assert_eq!(a.candidates.len(), 15);
    }

    #[test]
    fn a_study_made_by_the_twin_has_nothing_to_report() {
        for seed in 0..10 {
            assert!(study(seed).check().is_empty(), "seed {seed}");
        }
    }

    #[test]
    fn the_non_dominated_set_is_consistent_and_names_no_single_best() {
        let config = Config::default();
        let s = study(7);
        let places = s.placements(&config);
        assert_eq!(places.len(), 15);
        let front: Vec<&str> = places
            .iter()
            .filter(|p| p.non_dominated)
            .map(|p| p.name.as_str())
            .collect();
        assert!(!front.is_empty());
        assert!(front.len() < 15);
        for p in &places {
            assert!(!p.incomplete);
            if p.non_dominated {
                assert!(p.dominated_by.is_empty());
            } else {
                // whoever beats an alternative is itself either on the front or beaten by one on the front
                assert!(p.dominated_by.iter().any(|d| front.contains(&d.as_str())));
            }
        }
    }

    #[test]
    fn every_failure_is_reported_with_its_code() {
        let cases = [
            (Failure::Crash, Code::MissingMetric),
            (Failure::NoSource, Code::NoSource),
            (Failure::Diverged, Code::NotFinite),
        ];
        for (failure, code) in cases {
            let mut s = study(1);
            assert!(inject(&mut s, "d0003", "loss_w", failure), "{failure:?}");
            let found = s.check();
            assert!(found.iter().any(|f| f.code == code && f.subject == "d0003"));
        }
    }

    #[test]
    fn a_failure_on_something_that_does_not_exist_changes_nothing() {
        let mut s = study(1);
        let before = s.clone();
        assert!(!inject(&mut s, "nope", "loss_w", Failure::Crash));
        assert!(!inject(&mut s, "d0001", "nope", Failure::NoSource));
        assert!(!inject(&mut s, "d0001", "nope", Failure::Diverged));
        assert_eq!(s, before);
    }

    #[test]
    fn an_alternative_that_crashed_is_left_out_of_the_comparison() {
        let config = Config::default();
        let mut s = study(2);
        inject(&mut s, "d0005", "cost_eur", Failure::Crash);
        let places = s.placements(&config);
        let broken = places.iter().find(|p| p.name == "d0005").unwrap();
        assert!(broken.incomplete && !broken.non_dominated);
        assert!(places
            .iter()
            .all(|p| !p.dominated_by.iter().any(|d| d == "d0005")));
    }

    #[test]
    fn recovery_runs_the_failed_alternatives_again_and_the_study_is_clean() {
        let mut s = study(3);
        inject(&mut s, "d0002", "cost_eur", Failure::Crash);
        inject(&mut s, "d0002", "loss_w", Failure::Diverged);
        inject(&mut s, "d0009", "margin_pct", Failure::NoSource);
        assert!(!s.check().is_empty());
        let mut twin = SolverTwin::new(99);
        assert_eq!(recover(&mut s, &mut twin), 2);
        assert!(s.check().is_empty());
        // the parameters survive the second run
        assert_eq!(s.candidates[1].parameters["size_mm"], 2.0);
        assert_eq!(recover(&mut s, &mut twin), 0);
    }

    #[test]
    fn the_difference_between_two_alternatives_lists_what_the_sweep_changed() {
        let s = study(6);
        let changes = s.difference("d0001", "d0006").unwrap();
        assert!(changes
            .iter()
            .any(|c| c.kind == "parameter" && c.name == "gap_mm"));
        assert!(changes.iter().any(|c| c.kind != "parameter"));
        assert!(s.difference("d0001", "missing").is_err());
    }
}
