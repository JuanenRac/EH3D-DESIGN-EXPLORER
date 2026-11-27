# Changelog

All notable changes to this project are documented here.

## [Unreleased]

## [0.0.6] - Compare and front agree on equal figures and output stays on one line

- **`compare` agrees with `front` about equality:** with `equal_within` above 0, `compare` named a better side for figures that `front` treats as equal. It now says `equal`.
- **1 more test** of the Rust code, for the above.
- **A limit of the tolerance of equality is written down:** with `equal_within` above 0 (default 0) three alternatives can beat each other in a circle, so that none of them is in the non-dominated set and the set can be empty. `docs/STATUS.md` and the eight READMEs say it and one new test shows it. Nothing changes in how alternatives are compared.
- **A name read from a file can no longer forge a line of the output:** line breaks and control characters in the findings and the error messages are printed as spaces. What is compared, hashed and the exit codes do not change.
- **1 more test** of the Rust code, for the above.

## [0.0.5] - Repeated criteria and parameters are refused, and many more tests

- **A criterion, a figure or a parameter written twice is refused:** the last one took the place of the first in silence and changed which alternatives were dominated. This applies to studies and to sweep specifications.
- **A JSON array is no longer read as a study or a sweep:** only an object is.
- **8 more tests** of the Rust code: a study cut at every place, seeded grids with every combination once, the largest sweep and the tolerance never making two alternatives beat each other.
- **More tests, for the shared tools of the base and for the comparison of alternatives:** `tools/test_shared_tools.py` adds 65 tests that run the lessons linter, the workflow check, the CHANGELOG check of a pull request and the banner check on small inputs made by the test itself (findings, baselines, exit codes and the shape of what they print). The Rust code gets 15 more: on five seeded sets somebody is always on the front and nobody beats the one who beats them, the order the alternatives are written in does not change the placements, 600 alternatives are placed without a panic, an alternative that lacks a figure neither beats nor is beaten, the difference names the better one by the direction of each criterion and puts the parameters before the metrics, a text that is not a study is an error and never a panic, the most alternatives is read and one more is not, and a parameter that is not finite is named.
- **The register of requirements names a test for each row:** `docs/REQUIREMENTS.md` has one row for each capability with an identifier (`DEXP-001` on), its state (planned, prototype, implemented or validated), how far it is (done, partial, pending or blocked), its evidence (simulator, host-test or none), the test that shows it and its limits; a capability that is not written names the test that will show it. `tests/reqs.rs` reads the table and fails when a test does not exist, when a state does not agree with the progress or the evidence, when `docs/STATUS.md` does not say the same numbers, when a README does not link the documents or when a document holds a date or a private reference. `docs/SCOPE.md`, `docs/STATUS.md` and `docs/NOT_YET.md` are rewritten to the same shape and the eight READMEs link the five documents.
- The shared workflow checker (`tools/workflow_hygiene.py`) is the current one of the documentation hub: the filters under a trigger (branches, paths, types) are no longer read as events.

## [0.0.4] - Figure and name checks and stricter limits

- **The threat model says what the code already does:** the rows of `docs/THREAT_MODEL.md` that have a mitigation in the code and a test for it are marked `partial` and name the tests and what is still not covered; the rows without either stay `planned`.

- **`front` and `compare` run the check first:** with findings (repeated names, missing figures, figures without a source) they still print their answer but list the findings on the error output and exit with 1, and `compare` of two alternatives with the same parameters and figures says `# equal`. A test covers both.
- **An alternative with a metric that is not finite is incomplete:** it is flagged incomplete and takes no place on the front; before, a diverged value (not a number) left it marked as not dominated.
- **Simulation mode:** add `src/simulation.rs`, a host twin of the tools that compute the figures: a seeded solver (the same seed gives the same study), three injectable failures (crash, figure without a source, figure that is not finite) and a recovery that runs the failed alternatives again. Eight deterministic tests cover them. The manifest now says `evidence: simulator`; nothing has run against the real tools.
- **The check names a figure for a criterion the study does not declare:** `PHOENIX-DEXP-005` is reported when an alternative has a figure whose criterion is not in the study (a typo in the name used to leave the figure out of every comparison without a word).
- **The check names an alternative with an empty name:** `PHOENIX-DEXP-006` is reported for a name that is empty or only blanks, and a test keeps the codes different and in the order of their numbers.
- **A limit of zero bytes in the settings is refused:** `max_file_bytes = 0` is a bad value of its line, as `max_candidates = 0` already was, instead of refusing every input file.

## [0.0.3] - Split CI with a build cache and a pinned Rust toolchain

- **The shared validation tools are updated:** the validator also runs the workflow and lessons checks of the suite with their latest rules, including a check to run before every push.
- **The CI validation checks the workflows and the lessons of past CI failures against a baseline:** `tools/workflow_hygiene.py` and `tools/lessons_lint.py` (copies of the shared tools) run in the validation, and only findings that are not in `tools/workflow_hygiene_baseline.json` and `tools/lessons_lint_baseline.json` fail it, so what exists today stays as it is and a new one is caught. `python tools/workflow_hygiene.py --root . --detail` and `python tools/lessons_lint.py --repo .` list the findings.
- **The validation of the CI is the reusable one of the documentation hub:** the job `validate` calls `validate.yml` of EH3D-DOCS (pinned by commit) and the check of the CHANGELOG is a job of its own.
- **The Rust jobs keep a cache of the build:** `rust.yml` uses `Swatinem/rust-cache`, pinned by its commit, after the installation of Rust, so a run that changes little does not build the dependencies again.
- **The CI is split in two workflows:** the validation of the repository (manifest, documentation, pictures, tools) stays in `ci.yml` and the Rust jobs move to `rust.yml`, which does not run for a change that only touches Markdown files or pictures, so a change of the documentation no longer builds the code on two systems. The names of the jobs are the same.
- **The Rust toolchain is pinned:** `rust-toolchain.toml` fixes the version of Rust (the same one in every repository of the suite) with `rustfmt` and `clippy`, the workflows install that version and not `stable`, so a new release of Rust cannot turn the CI red by itself.

## [0.0.2] - First release: the library, the command line, the stable error codes and the tests

- Add the library: alternatives of a design with parameters and figures that name their source, compared across criteria with a direction, with the non-dominated set, who beats each of the others, and the difference between two alternatives; no single best alternative is named.
- Add the deterministic parametric sweep (a grid in a fixed order, with a limit on its size) and the stable codes `PHOENIX-DEXP-001` to `PHOENIX-DEXP-004`.
- Add the command line (`check`, `front`, `compare`, `sweep`) and a settings file; run format, lints and tests of the Rust code on Linux and Windows in the CI.
- **Brand:** Add the application icon and the logos of the program.
- **Shared checks updated:** the validator also compares the badge and the Version row of the README with the manifest, fails when one of its checks cannot run, and the CHANGELOG step of the workflow reads the base branch from the environment.
- **The workflows are hardened:** every action is pinned by its commit with its version in a comment, and the checkouts keep no credentials.

## [0.0.1] - The base of the repository

- **The base that every repository of the suite has:** the manifest, the README files in eight languages, this changelog, the CI workflow, the issue and pull-request templates, and the shared checks of EH3D-DOCS (the repository check and the drawers of the two README pictures).
- **A planned project, described honestly:** the scope, the requirements (all planned), the relations with other repositories, the threat model, the decisions that apply and the rules for writing code are in `docs/`; there is no code yet.
