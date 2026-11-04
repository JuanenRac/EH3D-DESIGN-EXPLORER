# EH3D-DESIGN-EXPLORER: status

## Current state

EH3D-DESIGN-EXPLORER is early code. Version 0.0.1 holds the base that every repository of the suite carries (manifest, README files in eight languages, changelog, CI, shared checks) and the written scope, requirements, dependencies, threat model and decisions. It also has a Rust library and a command line that read a set of alternatives of a design (parameters and figures that each name the tool and version that produced them) with its criteria, check it, show the place of every alternative without naming a single best one, list what differs between two alternatives and generate the grid of a parametric sweep in a fixed order. They have run only on data made by their own tests. Nothing has been run on a machine with real data, and no requirement is validated.

## Capabilities

The official state of each capability is in REQUIREMENTS.md: 5 requirements: 4 `prototype` and 1 `planned` (DEXP-005). This page never states a state that REQUIREMENTS.md does not.

## Open questions

* Which criteria the first version offers and how they are weighted by the user.

## Needs from other repositories

* EH3D-PHOENIX-PCB: the design model
* EH3D-DESIGN-VERIFIER: checks to use as criteria

## Known technical debt and omitted tests

19 tests of the Rust code and the 3 of the base; the Rust tests build their own studies and never touch real data. Not tested: very large studies. `cargo deny` is not run yet.
