// SPDX-License-Identifier: GPL-3.0-or-later
//! The parametric sweep: a plain grid. Parameters are taken in the order of their names and their values in the order given, the last parameter
//! changing fastest, so the same specification always gives the same alternatives in the same order. There is no randomness.

use crate::config::Config;
use crate::error::DexpError;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// The values of one parameter: a list or a range.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged, deny_unknown_fields)]
pub enum Values {
    /// The values, in this order.
    List(Vec<f64>),
    /// `start`, `start + step`, ... up to and including `stop`.
    Range {
        /// The first value.
        start: f64,
        /// The last value allowed.
        stop: f64,
        /// The step, above zero.
        step: f64,
    },
}

/// The specification of a sweep.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SweepSpec {
    /// The prefix of the names of the alternatives (`<prefix>0001`, ...).
    pub prefix: String,
    /// The parameters, with the unit in the name.
    pub parameters: BTreeMap<String, Values>,
}

/// One alternative of the sweep: a name and its parameters.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Generated {
    /// The name.
    pub name: String,
    /// The parameters.
    pub parameters: BTreeMap<String, f64>,
}

impl Values {
    fn expand(&self, name: &str, config: &Config) -> Result<Vec<f64>, DexpError> {
        let bad = |what: &str| DexpError::BadInput(format!("parameter `{name}`: {what}"));
        let values = match self {
            Values::List(v) => v.clone(),
            Values::Range { start, stop, step } => {
                if !start.is_finite()
                    || !stop.is_finite()
                    || !step.is_finite()
                    || *step <= 0.0
                    || stop < start
                {
                    return Err(bad(
                        "a range needs finite numbers, a step above zero and stop not below start",
                    ));
                }
                let count = ((stop - start) / step * (1.0 + 1e-12)).floor() + 1.0;
                if count > config.max_candidates as f64 {
                    return Err(DexpError::TooLarge {
                        what: "alternatives of the sweep",
                    });
                }
                (0..count as usize)
                    .map(|i| start + i as f64 * step)
                    .collect()
            }
        };
        if values.is_empty() {
            return Err(bad("no values"));
        }
        if values.iter().any(|v| !v.is_finite()) {
            return Err(bad("a value is not finite"));
        }
        Ok(values)
    }
}

/// The grid of a specification.
pub fn expand(spec: &SweepSpec, config: &Config) -> Result<Vec<Generated>, DexpError> {
    if spec.prefix.trim().is_empty() {
        return Err(DexpError::BadInput(
            "the prefix of the names is empty".into(),
        ));
    }
    if spec.parameters.is_empty() {
        return Err(DexpError::BadInput("the sweep has no parameters".into()));
    }
    let mut axes: Vec<(&String, Vec<f64>)> = Vec::new();
    let mut total: usize = 1;
    for (name, values) in &spec.parameters {
        let v = values.expand(name, config)?;
        total = total
            .checked_mul(v.len())
            .filter(|t| *t <= config.max_candidates)
            .ok_or(DexpError::TooLarge {
                what: "alternatives of the sweep",
            })?;
        axes.push((name, v));
    }
    let width = total.to_string().len().max(4);
    let mut out = Vec::with_capacity(total);
    for index in 0..total {
        let mut rest = index;
        let mut parameters = BTreeMap::new();
        for (name, values) in axes.iter().rev() {
            parameters.insert((*name).clone(), values[rest % values.len()]);
            rest /= values.len();
        }
        out.push(Generated {
            name: format!("{}{:0width$}", spec.prefix, index + 1, width = width),
            parameters,
        });
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spec(text: &str) -> SweepSpec {
        serde_json::from_str(text).unwrap()
    }

    #[test]
    fn the_grid_has_the_product_of_the_values_in_a_fixed_order() {
        let s = spec(r#"{"prefix":"c","parameters":{"b_mm":[1,2],"a_mm":[10,20,30]}}"#);
        let g = expand(&s, &Config::default()).unwrap();
        assert_eq!(g.len(), 6);
        assert_eq!(g[0].name, "c0001");
        // the parameters go in the order of their names (a_mm first), the last one changing fastest
        assert_eq!(
            (g[0].parameters["a_mm"], g[0].parameters["b_mm"]),
            (10.0, 1.0)
        );
        assert_eq!(
            (g[1].parameters["a_mm"], g[1].parameters["b_mm"]),
            (10.0, 2.0)
        );
        assert_eq!(
            (g[2].parameters["a_mm"], g[2].parameters["b_mm"]),
            (20.0, 1.0)
        );
        assert_eq!(
            (g[5].parameters["a_mm"], g[5].parameters["b_mm"]),
            (30.0, 2.0)
        );
    }

    #[test]
    fn the_same_specification_gives_the_same_alternatives() {
        let s = spec(
            r#"{"prefix":"x","parameters":{"w_mm":{"start":0.1,"stop":0.5,"step":0.1},"n":[1,2]}}"#,
        );
        assert_eq!(
            expand(&s, &Config::default()).unwrap(),
            expand(&s, &Config::default()).unwrap()
        );
    }

    #[test]
    fn a_range_includes_its_stop_without_float_drift() {
        let s = spec(r#"{"prefix":"x","parameters":{"w_mm":{"start":0.1,"stop":0.5,"step":0.1}}}"#);
        let g = expand(&s, &Config::default()).unwrap();
        assert_eq!(g.len(), 5);
        assert!((g[4].parameters["w_mm"] - 0.5).abs() < 1e-12);
    }

    #[test]
    fn bad_specifications_are_refused() {
        let cfg = Config::default();
        for text in [
            r#"{"prefix":"","parameters":{"a":[1]}}"#,
            r#"{"prefix":"x","parameters":{}}"#,
            r#"{"prefix":"x","parameters":{"a":[]}}"#,
            r#"{"prefix":"x","parameters":{"a":{"start":1,"stop":0,"step":1}}}"#,
            r#"{"prefix":"x","parameters":{"a":{"start":0,"stop":1,"step":0}}}"#,
            r#"{"prefix":"x","parameters":{"a":{"start":0,"stop":1,"step":-1}}}"#,
        ] {
            assert!(expand(&spec(text), &cfg).is_err(), "{text}");
        }
        assert!(serde_json::from_str::<SweepSpec>(
            r#"{"prefix":"x","parameters":{"a":[1]},"seed":3}"#
        )
        .is_err());
    }

    #[test]
    fn the_size_of_the_grid_is_limited_before_it_is_built() {
        let small = Config {
            max_candidates: 5,
            ..Config::default()
        };
        let s = spec(r#"{"prefix":"x","parameters":{"a":[1,2,3],"b":[1,2]}}"#);
        assert!(matches!(
            expand(&s, &small),
            Err(DexpError::TooLarge { .. })
        ));
        let huge = spec(r#"{"prefix":"x","parameters":{"a":{"start":0,"stop":1e18,"step":1}}}"#);
        assert!(matches!(
            expand(&huge, &Config::default()),
            Err(DexpError::TooLarge { .. })
        ));
    }
}
