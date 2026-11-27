// SPDX-License-Identifier: GPL-3.0-or-later
//! The command line. It is a thin layer: parsing the arguments and printing; the work lives in the other modules.
//!
//! Exit codes: 0 done and nothing found, 1 the study has findings (`check`, and also `front` and `compare`, which run the check first), 2 the input or the arguments are unusable.

use crate::config::Config;
use crate::error::DexpError;
use crate::explore::Study;
use crate::sweep::{expand, SweepSpec};
use std::fs;
use std::io::Write;
use std::path::Path;

const USAGE: &str = "\
eh3d-design-explorer [--config FILE] <command>

  check <study.json>                      list what makes a comparison less reliable: repeated names, missing figures, figures without a source
  front <study.json>                      the place of every alternative: non-dominated, or dominated and by whom (no single best is named)
  compare <study.json> <first> <second>   the parameters and figures that differ between two alternatives, and which side is better on each
  sweep <spec.json>                       the grid of alternatives of a parametric sweep, in a fixed order

`front` and `compare` run the check first: the findings go to the error output and the exit code is 1, but the answer is still printed.
Exit codes: 0 done, 1 findings, 2 unusable input.
";

fn read_text(path: &Path, limit: u64) -> Result<String, DexpError> {
    let io = |source| DexpError::Io {
        path: path.to_path_buf(),
        source,
    };
    if fs::metadata(path).map_err(io)?.len() > limit {
        return Err(DexpError::TooLarge { what: "input file" });
    }
    fs::read_to_string(path).map_err(io)
}

/// A text from an input file, fit to print on one line: control characters and the Unicode line and paragraph separators become spaces.
/// Only what is printed goes through here; what is compared or hashed does not.
fn one_line(text: &str) -> String {
    text.chars()
        .map(|c| {
            if c.is_control() || matches!(c, '\u{2028}' | '\u{2029}') {
                ' '
            } else {
                c
            }
        })
        .collect()
}

/// Run the command line with the given arguments (without the program name). Returns the exit code.
pub fn run(args: &[String], out: &mut dyn Write, err: &mut dyn Write) -> i32 {
    match dispatch(args, out, err) {
        Ok(code) => code,
        Err(None) => {
            let _ = err.write_all(USAGE.as_bytes());
            2
        }
        Err(Some(error)) => {
            let _ = writeln!(err, "error: {}", one_line(&error.to_string()));
            2
        }
    }
}

/// The findings of the check, one line each, on the error output; the exit code to use (1 with findings, 0 without).
fn report_findings(study: &Study, err: &mut dyn Write) -> i32 {
    let findings = study.check();
    for f in &findings {
        let subject = if f.subject.is_empty() {
            String::new()
        } else {
            format!(" [{}]", f.subject)
        };
        let line = format!("{}{}: {}", f.code.as_str(), subject, f.message);
        let _ = writeln!(err, "{}", one_line(&line));
    }
    if findings.is_empty() {
        0
    } else {
        let _ = writeln!(err, "# {} findings", findings.len());
        1
    }
}

fn dispatch(
    args: &[String],
    out: &mut dyn Write,
    err: &mut dyn Write,
) -> Result<i32, Option<DexpError>> {
    let mut args = args.to_vec();
    let mut config = Config::default();
    if args.first().map(String::as_str) == Some("--config") {
        if args.len() < 3 {
            return Err(None);
        }
        config = Config::parse(&read_text(Path::new(&args[1]), 1024 * 1024).map_err(Some)?)
            .map_err(Some)?;
        args.drain(..2);
    }
    let rest: Vec<&str> = args.iter().map(String::as_str).collect();
    let text = |path: &str| read_text(Path::new(path), config.max_file_bytes).map_err(Some);
    match rest.as_slice() {
        ["check", path] => {
            let study = Study::from_json(&text(path)?, &config).map_err(Some)?;
            let findings = study.check();
            for f in &findings {
                let subject = if f.subject.is_empty() {
                    String::new()
                } else {
                    format!(" [{}]", f.subject)
                };
                let line = format!("{}{}: {}", f.code.as_str(), subject, f.message);
                let _ = writeln!(out, "{}", one_line(&line));
            }
            let _ = writeln!(out, "# {} findings", findings.len());
            Ok(if findings.is_empty() { 0 } else { 1 })
        }
        ["front", path] => {
            let study = Study::from_json(&text(path)?, &config).map_err(Some)?;
            let _ = writeln!(out, "{}", pretty(&study.placements(&config)));
            Ok(report_findings(&study, err))
        }
        ["compare", path, first, second] => {
            let study = Study::from_json(&text(path)?, &config).map_err(Some)?;
            let mut changes = study.difference(first, second).map_err(Some)?;
            // A figure that `front` treats as equal must not be called better here.
            for change in &mut changes {
                if let ("metric", Some(x), Some(y)) = (change.kind, change.from, change.to) {
                    if (x - y).abs() <= config.equal_within * x.abs().max(y.abs()) {
                        change.better = "equal";
                    }
                }
            }
            let _ = writeln!(out, "{}", pretty(&changes));
            if changes.is_empty() {
                let _ = writeln!(
                    out,
                    "# equal: the two alternatives have the same parameters and figures"
                );
            }
            Ok(report_findings(&study, err))
        }
        ["sweep", path] => {
            let spec = SweepSpec::from_json(&text(path)?).map_err(Some)?;
            let grid = expand(&spec, &config).map_err(Some)?;
            let _ = writeln!(out, "{}", pretty(&grid));
            Ok(0)
        }
        _ => Err(None),
    }
}

fn pretty<T: serde::Serialize>(value: &T) -> String {
    serde_json::to_string_pretty(value).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_args(args: &[&str]) -> (i32, String, String) {
        let args: Vec<String> = args.iter().map(|s| s.to_string()).collect();
        let (mut out, mut err) = (Vec::new(), Vec::new());
        let code = run(&args, &mut out, &mut err);
        (
            code,
            String::from_utf8(out).unwrap(),
            String::from_utf8(err).unwrap(),
        )
    }

    fn file(dir: &tempfile::TempDir, name: &str, text: &str) -> String {
        let p = dir.path().join(name);
        fs::write(&p, text).unwrap();
        p.to_string_lossy().into_owned()
    }

    const STUDY: &str = r#"{"criteria":{"cost_eur":"min","temp_c":"min"},"candidates":[
      {"name":"cheap","parameters":{"w_mm":0.2},"metrics":{"cost_eur":{"value":1,"source":"t 1"},"temp_c":{"value":90,"source":"t 1"}}},
      {"name":"cool","parameters":{"w_mm":0.6},"metrics":{"cost_eur":{"value":3,"source":"t 1"},"temp_c":{"value":50,"source":"t 1"}}},
      {"name":"bad","parameters":{"w_mm":0.3},"metrics":{"cost_eur":{"value":4,"source":"t 1"},"temp_c":{"value":95,"source":"t 1"}}}]}"#;

    #[test]
    fn front_shows_every_alternative_and_names_no_winner() {
        let dir = tempfile::tempdir().unwrap();
        let s = file(&dir, "s.json", STUDY);
        let (code, out, _) = run_args(&["front", &s]);
        assert_eq!(code, 0);
        assert!(
            out.contains("\"name\": \"bad\"") && out.contains("\"dominated_by\""),
            "{out}"
        );
        assert_eq!(out.matches("\"non_dominated\": true").count(), 2, "{out}");
    }

    #[test]
    fn check_compare_and_sweep_work_on_files() {
        let dir = tempfile::tempdir().unwrap();
        let s = file(&dir, "s.json", STUDY);
        let (code, out, _) = run_args(&["check", &s]);
        assert_eq!(code, 0, "{out}");
        let (code, out, _) = run_args(&["compare", &s, "cheap", "cool"]);
        assert_eq!(code, 0);
        assert!(
            out.contains("\"better\": \"first\"") && out.contains("\"better\": \"second\""),
            "{out}"
        );
        let spec = file(
            &dir,
            "w.json",
            r#"{"prefix":"c","parameters":{"a_mm":[1,2]}}"#,
        );
        let (code, out, _) = run_args(&["sweep", &spec]);
        assert_eq!(code, 0);
        assert!(
            out.contains("\"c0001\"") && out.contains("\"c0002\""),
            "{out}"
        );
    }

    #[test]
    fn findings_exit_1_and_bad_input_exits_2() {
        let dir = tempfile::tempdir().unwrap();
        let dup = file(&dir, "d.json", &STUDY.replace("\"cool\"", "\"cheap\""));
        let (code, out, _) = run_args(&["check", &dup]);
        assert_eq!(code, 1);
        assert!(out.contains("PHOENIX-DEXP-001"), "{out}");
        let junk = file(&dir, "j.json", "{");
        assert_eq!(run_args(&["front", &junk]).0, 2);
        let s = file(&dir, "s.json", STUDY);
        assert_eq!(run_args(&["compare", &s, "cheap", "nobody"]).0, 2);
        assert_eq!(run_args(&["front", "missing.json"]).0, 2);
        assert_eq!(run_args(&[]).0, 2);
    }

    #[test]
    fn front_and_compare_run_the_check_and_exit_1_with_findings_but_still_answer() {
        let dir = tempfile::tempdir().unwrap();
        let dup = file(&dir, "d.json", &STUDY.replace("\"bad\"", "\"cheap\""));
        let (code, out, err) = run_args(&["front", &dup]);
        assert_eq!(code, 1);
        assert!(out.contains("\"name\""), "{out}");
        assert!(err.contains("PHOENIX-DEXP-001"), "{err}");
        let s = file(&dir, "s.json", STUDY);
        let (code, out, _) = run_args(&["compare", &s, "cheap", "cheap"]);
        assert_eq!(code, 0);
        assert!(out.contains("# equal"), "{out}");
    }

    #[test]
    fn the_settings_limit_the_sweep() {
        let dir = tempfile::tempdir().unwrap();
        let spec = file(
            &dir,
            "w.json",
            r#"{"prefix":"c","parameters":{"a":[1,2,3]}}"#,
        );
        let cfg = file(&dir, "c.cfg", "max_candidates = 2\n");
        let (code, _, err) = run_args(&["--config", &cfg, "sweep", &spec]);
        assert_eq!(code, 2);
        assert!(err.contains("limit exceeded"), "{err}");
    }

    #[test]
    fn compare_calls_no_side_better_when_the_front_treats_the_figures_as_equal() {
        let work = tempfile::tempdir().unwrap();
        let study = work.path().join("s.json");
        let settings = work.path().join("c.cfg");
        std::fs::write(
            &study,
            r#"{"criteria":{"mass_g":"min"},"candidates":[
                {"name":"a","metrics":{"mass_g":{"value":100.0,"source":"t 1"}}},
                {"name":"b","metrics":{"mass_g":{"value":100.5,"source":"t 1"}}}]}"#,
        )
        .unwrap();
        std::fs::write(&settings, "equal_within = 0.01\n").unwrap();
        let run = |args: &[&str]| {
            let args: Vec<String> = args.iter().map(|a| a.to_string()).collect();
            let (mut out, mut err) = (Vec::new(), Vec::new());
            let code = crate::cli::run(&args, &mut out, &mut err);
            (code, String::from_utf8(out).unwrap())
        };
        let path = study.to_str().unwrap();
        let (code, text) = run(&[
            "--config",
            settings.to_str().unwrap(),
            "compare",
            path,
            "a",
            "b",
        ]);
        assert_eq!(code, 0, "{text}");
        assert!(text.contains(r#""better": "equal""#), "{text}");
        let (code, text) = run(&["compare", path, "a", "b"]);
        assert_eq!(code, 0, "{text}");
        assert!(text.contains(r#""better": "first""#), "{text}");
    }

    #[test]
    fn names_read_from_the_input_cannot_start_a_line_of_their_own() {
        let work = tempfile::tempdir().unwrap();
        let file = |name: &str, text: &str| {
            let path = work.path().join(name);
            std::fs::write(&path, text).unwrap();
            path.to_string_lossy().into_owned()
        };
        let call = |list: &[&str]| {
            let args: Vec<String> = list.iter().map(|s| s.to_string()).collect();
            let (mut out, mut err) = (Vec::new(), Vec::new());
            let code = run(&args, &mut out, &mut err);
            let out = String::from_utf8(out).unwrap();
            let err = String::from_utf8(err).unwrap();
            for c in out.chars().chain(err.chars()) {
                assert!(
                    c == '\n' || !(c.is_control() || c == '\u{2028}' || c == '\u{2029}'),
                    "{c:?} in {out}{err}"
                );
            }
            (code, out, err)
        };
        let twice = file(
            "twice.json",
            r#"{"criteria":{"cost_eur":"min"},"candidates":[{"name":"a\nforged"},{"name":"a\nforged"}]}"#,
        );
        let (code, out, _) = call(&["check", &twice]);
        assert_eq!(code, 1, "{out}");
        assert!(out.contains("forged"), "{out}");
        for line in out.lines() {
            assert!(
                line.starts_with("PHOENIX-") || line.starts_with("# "),
                "{out}"
            );
        }
        let (code, _, err) = call(&["front", &twice]);
        assert_eq!(code, 1, "{err}");
        assert!(err.contains("forged"), "{err}");
        for line in err.lines() {
            assert!(
                line.starts_with("PHOENIX-") || line.starts_with("# "),
                "{err}"
            );
        }
        let unknown = file("unknown.json", r#"{"x\nforged\u2028":1}"#);
        let (code, _, err) = call(&["check", &unknown]);
        assert_eq!(code, 2, "{err}");
        assert_eq!(err.lines().count(), 1, "{err}");
        assert!(err.contains("forged"), "{err}");
    }
}
