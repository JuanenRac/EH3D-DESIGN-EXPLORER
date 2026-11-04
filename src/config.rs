// SPDX-License-Identifier: GPL-3.0-or-later
//! The settings of the library. Every tuned value is a line of the settings file; a missing line keeps the conservative default.

use crate::error::DexpError;

/// The settings.
#[derive(Debug, Clone, PartialEq)]
pub struct Config {
    /// The most bytes an input file may have.
    pub max_file_bytes: u64,
    /// The most alternatives a file may hold or a sweep may produce.
    pub max_candidates: usize,
    /// The most criteria of a comparison.
    pub max_criteria: usize,
    /// Metrics that differ by no more than this fraction of the larger absolute value count as equal when judging domination (0: exact).
    pub equal_within: f64,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            max_file_bytes: 16 * 1024 * 1024,
            max_candidates: 10_000,
            max_criteria: 64,
            equal_within: 0.0,
        }
    }
}

impl Config {
    /// Parse a settings file (`key = value` lines, `#` comments).
    pub fn parse(text: &str) -> Result<Config, DexpError> {
        let mut config = Config::default();
        for (index, raw) in text.lines().enumerate() {
            let line = index + 1;
            let raw = raw.trim();
            if raw.is_empty() || raw.starts_with('#') {
                continue;
            }
            let (key, value) = raw
                .split_once('=')
                .ok_or(DexpError::ConfigSyntax { line })?;
            let (key, value) = (key.trim(), value.trim());
            let bad = || DexpError::ConfigBadValue {
                line,
                key: key.to_string(),
            };
            match key {
                "max_file_bytes" => config.max_file_bytes = value.parse().map_err(|_| bad())?,
                "max_candidates" | "max_criteria" => {
                    let n: usize = value.parse().map_err(|_| bad())?;
                    if n == 0 {
                        return Err(bad());
                    }
                    if key == "max_candidates" {
                        config.max_candidates = n;
                    } else {
                        config.max_criteria = n;
                    }
                }
                "equal_within" => {
                    let n: f64 = value.parse().map_err(|_| bad())?;
                    if !n.is_finite() || !(0.0..=0.5).contains(&n) {
                        return Err(bad());
                    }
                    config.equal_within = n;
                }
                _ => {
                    return Err(DexpError::ConfigUnknownKey {
                        line,
                        key: key.to_string(),
                    })
                }
            }
        }
        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_exact() {
        assert_eq!(Config::parse("").unwrap().equal_within, 0.0);
    }

    #[test]
    fn lines_change_values_and_bad_lines_are_refused() {
        let c = Config::parse("max_candidates = 5\nequal_within = 0.01\n").unwrap();
        assert_eq!((c.max_candidates, c.equal_within), (5, 0.01));
        assert!(matches!(
            Config::parse("x\n"),
            Err(DexpError::ConfigSyntax { line: 1 })
        ));
        assert!(matches!(
            Config::parse("a = 1\n"),
            Err(DexpError::ConfigUnknownKey { line: 1, .. })
        ));
        for bad in [
            "max_candidates = 0\n",
            "equal_within = 0.9\n",
            "equal_within = nan\n",
            "max_criteria = x\n",
        ] {
            assert!(
                matches!(
                    Config::parse(bad),
                    Err(DexpError::ConfigBadValue { line: 1, .. })
                ),
                "{bad:?}"
            );
        }
    }
}
