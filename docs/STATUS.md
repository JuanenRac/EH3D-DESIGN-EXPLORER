# EH3D-DESIGN-EXPLORER: status

## Current state

EH3D-DESIGN-EXPLORER is early code on a computer: a Rust library and a command line (`check`, `front`, `compare`, `sweep`) that read a set of alternatives of a design with its criteria, check it, show the place of every alternative without naming a single best one, list what differs between two alternatives and generate the grid of a parametric sweep in a fixed order. A seeded twin of the tools that compute the figures injects three failures. All of it has run only on data made by its own tests; the comparison of branches is not written and no requirement is validated.

## Capabilities

The official state of each capability is in `REQUIREMENTS.md`. The register has 31 requirements: 25 implemented, 4 prototype, 2 planned and 0 validated. By progress: 25 done, 4 partial, 1 pending and 1 blocked. This page never states a state that `REQUIREMENTS.md` does not.

## Open questions

* Which criteria the first version offers and how the user weighs them.
* How a branch of the design history becomes a study.

## What it needs from other repositories

* EH3D-PHOENIX-PCB: the design model.
* EH3D-DESIGN-VERIFIER: the checks to use as criteria.
* EH3D-DESIGN-HISTORY: the branches to compare.

## Known technical debt and omitted tests

* 57 tests of the Rust code (nine of them keep this register true) and the 68 of the base (65 of them test the shared tools of the base); the Rust tests build their own studies and never touch real data.
* Eight of the tests run the simulation (`src/simulation.rs`): the three failures that can be injected (a crash, a figure without its source, a figure that is not finite), the comparison that leaves a failed alternative out and the recovery.
* Not tested: very large studies.
* `cargo deny` is not run yet.
