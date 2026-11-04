// SPDX-License-Identifier: GPL-3.0-or-later
//! The alternatives, the checks, the non-dominated set and the difference between two alternatives.

use crate::config::Config;
use crate::error::DexpError;
use crate::Code;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// Whether a smaller or a larger value of a criterion is better.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Goal {
    /// Smaller is better (cost, size, losses, temperature).
    Min,
    /// Larger is better.
    Max,
}

/// One measured or computed figure and where it comes from.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Metric {
    /// The value, in the unit the name of the criterion ends in.
    pub value: f64,
    /// The tool and its version that produced the value.
    #[serde(default)]
    pub source: String,
}

/// One alternative.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Candidate {
    /// Its name, unique in the set.
    pub name: String,
    /// The parameters it is made from, with the unit in the name.
    #[serde(default)]
    pub parameters: BTreeMap<String, f64>,
    /// Its figures by criterion.
    #[serde(default)]
    pub metrics: BTreeMap<String, Metric>,
}

/// A set of alternatives and the criteria they are compared on.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Study {
    /// The criteria and their direction.
    pub criteria: BTreeMap<String, Goal>,
    /// The alternatives.
    pub candidates: Vec<Candidate>,
}

/// Something wrong in a study.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    /// The code.
    pub code: Code,
    /// The alternative it is about (empty for the study).
    pub subject: String,
    /// What is wrong, in English.
    pub message: String,
}

/// The place of an alternative in the comparison.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Placement {
    /// The alternative.
    pub name: String,
    /// True when no complete alternative beats it on every criterion.
    pub non_dominated: bool,
    /// The alternatives that beat it (empty when it is non-dominated).
    pub dominated_by: Vec<String>,
    /// True when it lacks a metric and so could not be compared.
    pub incomplete: bool,
}

/// One difference between two alternatives.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Change {
    /// `parameter` or `metric`.
    pub kind: &'static str,
    /// The name of the parameter or criterion.
    pub name: String,
    /// The value in the first alternative (None when it has none).
    pub from: Option<f64>,
    /// The value in the second alternative (None when it has none).
    pub to: Option<f64>,
    /// For a metric: which alternative is better on it (`first`, `second` or `equal`); empty for a parameter.
    pub better: &'static str,
}

impl Study {
    /// Read a study from JSON (unknown fields are refused) and apply the size limits.
    pub fn from_json(text: &str, config: &Config) -> Result<Study, DexpError> {
        let study: Study =
            serde_json::from_str(text).map_err(|e| DexpError::BadJson(e.to_string()))?;
        if study.candidates.len() > config.max_candidates {
            return Err(DexpError::TooLarge {
                what: "alternatives",
            });
        }
        if study.criteria.len() > config.max_criteria {
            return Err(DexpError::TooLarge { what: "criteria" });
        }
        if study.criteria.is_empty() {
            return Err(DexpError::BadInput("the study has no criteria".into()));
        }
        Ok(study)
    }

    fn complete(&self, c: &Candidate) -> bool {
        self.criteria.keys().all(|k| c.metrics.contains_key(k))
    }

    /// Everything that makes a comparison less reliable: repeated names, missing metrics, metrics without a source, numbers that are not finite.
    pub fn check(&self) -> Vec<Finding> {
        let mut out = Vec::new();
        let mut seen = BTreeSet::new();
        for c in &self.candidates {
            if !seen.insert(c.name.as_str()) {
                out.push(Finding {
                    code: Code::DuplicateName,
                    subject: c.name.clone(),
                    message: "the name is used by more than one alternative".into(),
                });
            }
            for criterion in self.criteria.keys() {
                match c.metrics.get(criterion) {
                    None => out.push(Finding {
                        code: Code::MissingMetric,
                        subject: c.name.clone(),
                        message: format!("no figure for the criterion `{criterion}`"),
                    }),
                    Some(m) => {
                        if m.source.trim().is_empty() {
                            out.push(Finding {
                                code: Code::NoSource,
                                subject: c.name.clone(),
                                message: format!(
                                    "the figure for `{criterion}` names no tool and version"
                                ),
                            });
                        }
                    }
                }
            }
            let bad = c
                .parameters
                .iter()
                .filter(|(_, v)| !v.is_finite())
                .map(|(k, _)| k.as_str())
                .chain(
                    c.metrics
                        .iter()
                        .filter(|(_, m)| !m.value.is_finite())
                        .map(|(k, _)| k.as_str()),
                );
            for name in bad {
                out.push(Finding {
                    code: Code::NotFinite,
                    subject: c.name.clone(),
                    message: format!("`{name}` is not a finite number"),
                });
            }
        }
        out
    }

    /// True when `a` is at least as good as `b` on every criterion and strictly better on one.
    fn beats(&self, a: &Candidate, b: &Candidate, config: &Config) -> bool {
        let mut strictly = false;
        for (criterion, goal) in &self.criteria {
            let (Some(x), Some(y)) = (a.metrics.get(criterion), b.metrics.get(criterion)) else {
                return false;
            };
            let (x, y) = (x.value, y.value);
            if !x.is_finite() || !y.is_finite() {
                return false;
            }
            let scale = x.abs().max(y.abs());
            if (x - y).abs() <= config.equal_within * scale {
                continue;
            }
            let better = match goal {
                Goal::Min => x < y,
                Goal::Max => x > y,
            };
            if !better {
                return false;
            }
            strictly = true;
        }
        strictly
    }

    /// The place of every alternative, ordered by name. Alternatives that lack a metric are marked incomplete and never take part in the comparison.
    pub fn placements(&self, config: &Config) -> Vec<Placement> {
        let mut all: Vec<&Candidate> = self.candidates.iter().collect();
        all.sort_by(|a, b| a.name.cmp(&b.name));
        let usable: Vec<&Candidate> = all.iter().copied().filter(|c| self.complete(c)).collect();
        all.iter()
            .map(|c| {
                if !self.complete(c) {
                    return Placement {
                        name: c.name.clone(),
                        non_dominated: false,
                        dominated_by: Vec::new(),
                        incomplete: true,
                    };
                }
                let dominated_by: Vec<String> = usable
                    .iter()
                    .filter(|o| !std::ptr::eq(**o, *c) && self.beats(o, c, config))
                    .map(|o| o.name.clone())
                    .collect();
                Placement {
                    name: c.name.clone(),
                    non_dominated: dominated_by.is_empty(),
                    dominated_by,
                    incomplete: false,
                }
            })
            .collect()
    }

    /// The changes between two alternatives by name: every parameter and every metric that differs.
    pub fn difference(&self, first: &str, second: &str) -> Result<Vec<Change>, DexpError> {
        let find = |n: &str| {
            self.candidates
                .iter()
                .find(|c| c.name == n)
                .ok_or_else(|| DexpError::BadInput(format!("there is no alternative named `{n}`")))
        };
        let (a, b) = (find(first)?, find(second)?);
        let mut out = Vec::new();
        let keys: BTreeSet<&String> = a.parameters.keys().chain(b.parameters.keys()).collect();
        for k in keys {
            let (x, y) = (a.parameters.get(k).copied(), b.parameters.get(k).copied());
            if x != y {
                out.push(Change {
                    kind: "parameter",
                    name: k.clone(),
                    from: x,
                    to: y,
                    better: "",
                });
            }
        }
        for (criterion, goal) in &self.criteria {
            let (x, y) = (
                a.metrics.get(criterion).map(|m| m.value),
                b.metrics.get(criterion).map(|m| m.value),
            );
            if x == y {
                continue;
            }
            let better = match (x, y) {
                (Some(x), Some(y)) => match (goal, x < y) {
                    (Goal::Min, true) | (Goal::Max, false) => "first",
                    _ => "second",
                },
                _ => "",
            };
            out.push(Change {
                kind: "metric",
                name: criterion.clone(),
                from: x,
                to: y,
                better,
            });
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn m(v: f64) -> Metric {
        Metric {
            value: v,
            source: "toollab 1.0".into(),
        }
    }

    fn cand(name: &str, cost: f64, temp: f64) -> Candidate {
        Candidate {
            name: name.into(),
            parameters: BTreeMap::from([("width_mm".to_string(), cost)]),
            metrics: BTreeMap::from([
                ("cost_eur".to_string(), m(cost)),
                ("temp_c".to_string(), m(temp)),
            ]),
        }
    }

    fn study(cands: Vec<Candidate>) -> Study {
        Study {
            criteria: BTreeMap::from([
                ("cost_eur".to_string(), Goal::Min),
                ("temp_c".to_string(), Goal::Min),
            ]),
            candidates: cands,
        }
    }

    #[test]
    fn a_trade_off_keeps_both_and_a_dominated_one_names_who_beats_it() {
        let s = study(vec![
            cand("cheap", 1.0, 90.0),
            cand("cool", 3.0, 50.0),
            cand("bad", 4.0, 95.0),
        ]);
        let p = s.placements(&Config::default());
        let by = |n: &str| p.iter().find(|x| x.name == n).unwrap().clone();
        assert!(by("cheap").non_dominated && by("cool").non_dominated);
        assert!(!by("bad").non_dominated);
        assert_eq!(by("bad").dominated_by, vec!["cheap", "cool"]);
    }

    #[test]
    fn equal_alternatives_do_not_beat_each_other() {
        let s = study(vec![cand("a", 1.0, 50.0), cand("b", 1.0, 50.0)]);
        assert!(s
            .placements(&Config::default())
            .iter()
            .all(|p| p.non_dominated));
    }

    #[test]
    fn the_direction_of_a_criterion_matters() {
        let mut s = study(vec![cand("a", 1.0, 50.0), cand("b", 2.0, 60.0)]);
        s.criteria.insert("temp_c".into(), Goal::Max);
        // a is cheaper, b is hotter and hotter is better: neither beats the other
        assert!(s
            .placements(&Config::default())
            .iter()
            .all(|p| p.non_dominated));
    }

    #[test]
    fn a_tolerance_for_equality_changes_what_counts_as_better() {
        let s = study(vec![cand("a", 1.0, 100.0), cand("b", 1.0, 100.5)]);
        assert!(!s.placements(&Config::default())[1].non_dominated);
        let cfg = Config {
            equal_within: 0.01,
            ..Config::default()
        };
        assert!(s.placements(&cfg).iter().all(|p| p.non_dominated));
    }

    #[test]
    fn an_incomplete_alternative_is_not_compared_and_is_flagged() {
        let mut half = cand("half", 0.1, 10.0);
        half.metrics.remove("temp_c");
        let s = study(vec![half, cand("full", 5.0, 80.0)]);
        let p = s.placements(&Config::default());
        assert!(p[1].incomplete && !p[1].non_dominated);
        assert!(
            p[0].non_dominated,
            "the incomplete one must not hide the full one"
        );
        assert!(s
            .check()
            .iter()
            .any(|f| f.code == Code::MissingMetric && f.subject == "half"));
    }

    #[test]
    fn check_finds_duplicates_missing_sources_and_bad_numbers() {
        let mut a = cand("a", 1.0, 50.0);
        a.metrics.get_mut("cost_eur").unwrap().source.clear();
        let mut b = cand("a", 1.0, 50.0);
        b.parameters.insert("x_mm".into(), f64::NAN);
        let codes: Vec<Code> = study(vec![a, b]).check().iter().map(|f| f.code).collect();
        for c in [Code::DuplicateName, Code::NoSource, Code::NotFinite] {
            assert!(codes.contains(&c), "{c:?}");
        }
        assert!(study(vec![cand("ok", 1.0, 2.0)]).check().is_empty());
    }

    #[test]
    fn the_difference_lists_parameters_and_metrics_with_the_better_side() {
        let s = study(vec![cand("a", 1.0, 90.0), cand("b", 3.0, 50.0)]);
        let d = s.difference("a", "b").unwrap();
        assert_eq!(d.len(), 3);
        let cost = d.iter().find(|c| c.name == "cost_eur").unwrap();
        assert_eq!(cost.better, "first");
        let temp = d.iter().find(|c| c.name == "temp_c").unwrap();
        assert_eq!(temp.better, "second");
        assert!(d
            .iter()
            .any(|c| c.kind == "parameter" && c.name == "width_mm"));
        assert!(s.difference("a", "a").unwrap().is_empty());
        assert!(s.difference("a", "zzz").is_err());
    }

    #[test]
    fn json_is_strict_and_limited() {
        let cfg = Config::default();
        let ok = r#"{"criteria":{"cost_eur":"min"},"candidates":[{"name":"a","metrics":{"cost_eur":{"value":1,"source":"t 1"}}}]}"#;
        assert!(Study::from_json(ok, &cfg).is_ok());
        assert!(Study::from_json(&ok.replace("candidates", "cands"), &cfg).is_err());
        assert!(Study::from_json(r#"{"criteria":{},"candidates":[]}"#, &cfg).is_err());
        let small = Config {
            max_candidates: 1,
            ..Config::default()
        };
        let two = r#"{"criteria":{"c":"max"},"candidates":[{"name":"a"},{"name":"b"}]}"#;
        assert!(matches!(
            Study::from_json(two, &small),
            Err(DexpError::TooLarge { .. })
        ));
        assert!(Study::from_json(&ok.replace("min", "smallest"), &cfg).is_err());
    }
}
