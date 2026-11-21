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

    /// An alternative takes part in the comparison only with every criterion measured by a number that is finite: one that diverged would otherwise be beaten by nobody.
    fn complete(&self, c: &Candidate) -> bool {
        self.criteria
            .keys()
            .all(|k| c.metrics.get(k).is_some_and(|m| m.value.is_finite()))
    }

    /// Everything that makes a comparison less reliable: repeated names, missing metrics, metrics without a source, numbers that are not finite.
    pub fn check(&self) -> Vec<Finding> {
        let mut out = Vec::new();
        let mut seen = BTreeSet::new();
        for c in &self.candidates {
            if c.name.trim().is_empty() {
                out.push(Finding {
                    code: Code::EmptyName,
                    subject: c.name.clone(),
                    message: "the name of the alternative is empty".into(),
                });
            }
            for criterion in c.metrics.keys() {
                if !self.criteria.contains_key(criterion) {
                    out.push(Finding {
                        code: Code::UnknownCriterion,
                        subject: c.name.clone(),
                        message: format!(
                            "a figure for `{criterion}`, which the study does not declare as a criterion"
                        ),
                    });
                }
            }
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
    fn a_figure_for_a_criterion_the_study_does_not_declare_is_a_finding() {
        let mut c = cand("typo", 1.0, 50.0);
        c.metrics.insert("cost_eru".to_string(), m(2.0));
        let findings = study(vec![c, cand("fine", 2.0, 40.0)]).check();
        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(findings[0].code, Code::UnknownCriterion);
        assert_eq!(findings[0].subject, "typo");
        assert!(findings[0].message.contains("cost_eru"));
        assert_eq!(findings[0].code.as_str(), "PHOENIX-DEXP-005");
    }

    #[test]
    fn a_name_that_is_empty_or_only_blanks_is_a_finding() {
        for name in ["", "   ", "\t"] {
            let findings = study(vec![cand(name, 1.0, 50.0)]).check();
            assert_eq!(findings.len(), 1, "{name:?}: {findings:?}");
            assert_eq!(findings[0].code, Code::EmptyName);
            assert_eq!(findings[0].code.as_str(), "PHOENIX-DEXP-006");
        }
        assert!(study(vec![cand("a", 1.0, 50.0)]).check().is_empty());
    }

    #[test]
    fn the_codes_are_all_different_and_follow_the_numbers() {
        let all = [
            Code::DuplicateName,
            Code::MissingMetric,
            Code::NoSource,
            Code::NotFinite,
            Code::UnknownCriterion,
            Code::EmptyName,
        ];
        for (i, code) in all.iter().enumerate() {
            assert_eq!(code.as_str(), format!("PHOENIX-DEXP-{:03}", i + 1));
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
    fn an_alternative_with_a_metric_that_is_not_finite_is_incomplete_and_never_on_the_front() {
        let mut diverged = cand("diverged", 0.1, 10.0);
        diverged.metrics.insert("temp_c".to_string(), m(f64::NAN));
        let s = study(vec![diverged, cand("full", 5.0, 80.0)]);
        let p = s.placements(&Config::default());
        assert!(p[0].incomplete && !p[0].non_dominated, "{p:?}");
        assert!(p[1].non_dominated && !p[1].incomplete, "{p:?}");
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

    fn lcg(seed: &mut u64) -> f64 {
        *seed = (*seed * 1_103_515_245 + 12_345) % 2_147_483_648;
        ((*seed >> 8) % 1000) as f64
    }

    fn seeded(count: usize, seed: u64) -> Study {
        let mut state = seed;
        let cands = (0..count)
            .map(|n| {
                let (a, b) = (lcg(&mut state), lcg(&mut state));
                cand(&format!("c{n:03}"), a, b)
            })
            .collect();
        study(cands)
    }

    #[test]
    fn in_a_seeded_set_somebody_is_always_on_the_front_and_nobody_beats_the_one_who_beats_them() {
        for seed in [1_u64, 2, 3, 4, 5] {
            let p = seeded(40, seed).placements(&Config::default());
            assert!(p.iter().any(|x| x.non_dominated), "seed {seed}");
            for x in &p {
                assert_eq!(x.non_dominated, x.dominated_by.is_empty());
                assert!(!x.dominated_by.contains(&x.name));
                for beater in &x.dominated_by {
                    let other = p.iter().find(|o| &o.name == beater).unwrap();
                    assert!(!other.dominated_by.contains(&x.name), "{seed}");
                }
            }
        }
    }

    #[test]
    fn the_order_in_which_the_alternatives_are_written_does_not_change_the_placements() {
        let forward = seeded(30, 9);
        let mut backward = forward.clone();
        backward.candidates.reverse();
        assert_eq!(
            forward.placements(&Config::default()),
            backward.placements(&Config::default())
        );
    }

    #[test]
    fn the_placements_come_sorted_by_the_name_of_the_alternative() {
        let p = seeded(25, 11).placements(&Config::default());
        let names: Vec<&String> = p.iter().map(|x| &x.name).collect();
        let mut sorted = names.clone();
        sorted.sort();
        assert_eq!(names, sorted);
    }

    #[test]
    fn six_hundred_alternatives_are_placed_without_a_panic() {
        let s = seeded(600, 21);
        let p = s.placements(&Config::default());
        assert_eq!(p.len(), 600);
        assert!(p.iter().filter(|x| x.non_dominated).count() >= 1);
    }

    #[test]
    fn a_single_alternative_is_on_the_front_and_a_study_without_alternatives_places_nobody() {
        let one = study(vec![cand("only", 1.0, 1.0)]).placements(&Config::default());
        assert_eq!(one.len(), 1);
        assert!(one[0].non_dominated && !one[0].incomplete);
        let none = study(vec![]);
        assert!(none.placements(&Config::default()).is_empty());
        assert!(none.check().is_empty());
    }

    #[test]
    fn an_alternative_that_lacks_a_figure_neither_beats_nor_is_beaten() {
        let mut lacking = cand("lacking", 0.1, 1.0);
        lacking.metrics.remove("temp_c");
        let s = study(vec![lacking, cand("full", 5.0, 80.0)]);
        let p = s.placements(&Config::default());
        let lacking = p.iter().find(|x| x.name == "lacking").unwrap();
        let full = p.iter().find(|x| x.name == "full").unwrap();
        assert!(lacking.incomplete && !lacking.non_dominated);
        assert!(full.non_dominated && full.dominated_by.is_empty());
    }

    #[test]
    fn a_difference_between_an_alternative_and_itself_is_empty() {
        let s = study(vec![cand("a", 1.0, 50.0)]);
        assert!(s.difference("a", "a").unwrap().is_empty());
    }

    #[test]
    fn a_difference_with_a_name_that_does_not_exist_is_an_error_whichever_side_it_is_on() {
        let s = study(vec![cand("a", 1.0, 50.0)]);
        assert!(matches!(
            s.difference("a", "x"),
            Err(DexpError::BadInput(_))
        ));
        assert!(matches!(
            s.difference("x", "a"),
            Err(DexpError::BadInput(_))
        ));
    }

    #[test]
    fn the_difference_says_which_one_is_better_by_the_direction_of_the_criterion() {
        let mut s = study(vec![cand("a", 1.0, 50.0), cand("b", 2.0, 60.0)]);
        s.criteria.insert("temp_c".into(), Goal::Max);
        let changes = s.difference("a", "b").unwrap();
        let better = |name: &str| {
            changes
                .iter()
                .find(|c| c.name == name && c.kind == "metric")
                .unwrap()
                .better
        };
        assert_eq!(better("cost_eur"), "first");
        assert_eq!(better("temp_c"), "second");
    }

    #[test]
    fn a_parameter_that_only_one_alternative_has_is_a_difference_with_a_missing_side() {
        let mut b = cand("b", 1.0, 50.0);
        b.parameters.insert("height_mm".into(), 3.0);
        let s = study(vec![cand("a", 1.0, 50.0), b]);
        let changes = s.difference("a", "b").unwrap();
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].kind, "parameter");
        assert_eq!((changes[0].from, changes[0].to), (None, Some(3.0)));
    }

    #[test]
    fn the_parameters_come_before_the_metrics_in_a_difference() {
        let s = study(vec![cand("a", 1.0, 50.0), cand("b", 2.0, 60.0)]);
        let kinds: Vec<&str> = s
            .difference("a", "b")
            .unwrap()
            .iter()
            .map(|c| c.kind)
            .collect();
        assert_eq!(kinds, ["parameter", "metric", "metric"]);
    }

    #[test]
    fn a_text_that_is_not_a_study_is_an_error_and_never_a_panic() {
        let config = Config::default();
        for text in ["", " ", "null", "[]", "12", "{", "{}", "\u{feff}{}"] {
            assert!(
                matches!(Study::from_json(text, &config), Err(DexpError::BadJson(_))),
                "{text:?}"
            );
        }
    }

    #[test]
    fn a_study_without_criteria_is_refused_and_the_most_alternatives_is_read_and_one_more_is_not() {
        let config = Config {
            max_candidates: 2,
            ..Config::default()
        };
        let empty = r#"{"criteria":{},"candidates":[]}"#;
        assert!(matches!(
            Study::from_json(empty, &config),
            Err(DexpError::BadInput(_))
        ));
        let two =
            serde_json::to_string(&study(vec![cand("a", 1.0, 1.0), cand("b", 2.0, 2.0)])).unwrap();
        assert!(Study::from_json(&two, &config).is_ok());
        let three = serde_json::to_string(&study(vec![
            cand("a", 1.0, 1.0),
            cand("b", 2.0, 2.0),
            cand("c", 3.0, 3.0),
        ]))
        .unwrap();
        assert!(matches!(
            Study::from_json(&three, &config),
            Err(DexpError::TooLarge { .. })
        ));
    }

    #[test]
    fn a_goal_is_written_in_lower_case_and_a_goal_that_is_not_one_is_refused() {
        assert_eq!(serde_json::to_string(&Goal::Min).unwrap(), "\"min\"");
        assert_eq!(serde_json::to_string(&Goal::Max).unwrap(), "\"max\"");
        let text = r#"{"criteria":{"cost_eur":"smallest"},"candidates":[]}"#;
        assert!(matches!(
            Study::from_json(text, &Config::default()),
            Err(DexpError::BadJson(_))
        ));
    }

    #[test]
    fn a_parameter_that_is_not_finite_is_one_finding_and_names_the_parameter() {
        let mut c = cand("a", 1.0, 50.0);
        c.parameters.insert("height_mm".into(), f64::INFINITY);
        let findings = study(vec![c]).check();
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].code, Code::NotFinite);
        assert!(findings[0].message.contains("height_mm"));
    }
}
