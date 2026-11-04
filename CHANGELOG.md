# Changelog

All notable changes to this project are documented here.

## [Unreleased]
- Add the library: alternatives of a design with parameters and figures that name their source, compared across criteria with a direction, with the non-dominated set, who beats each of the others, and the difference between two alternatives; no single best alternative is named.
- Add the deterministic parametric sweep (a grid in a fixed order, with a limit on its size) and the stable codes `PHOENIX-DEXP-001` to `PHOENIX-DEXP-004`.
- Add the command line (`check`, `front`, `compare`, `sweep`) and a settings file; run format, lints and tests of the Rust code on Linux and Windows in the CI.
- **Brand:** Add the application icon and the logos of the program.
- **Shared checks updated:** the validator also compares the badge and the Version row of the README with the manifest, fails when one of its checks cannot run, and the CHANGELOG step of the workflow reads the base branch from the environment.
- **The workflows are hardened:** every action is pinned by its commit with its version in a comment, and the checkouts keep no credentials.

## [0.0.1] - The base of the repository

- **The base that every repository of the suite has:** the manifest, the README files in eight languages, this changelog, the CI workflow, the issue and pull-request templates, and the shared checks of EH3D-DOCS (the repository check and the drawers of the two README pictures).
- **A planned project, described honestly:** the scope, the requirements (all planned), the relations with other repositories, the threat model, the decisions that apply and the rules for writing code are in `docs/`; there is no code yet.
