# Changelog

All notable changes to this project are documented here.

## [Unreleased]

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
