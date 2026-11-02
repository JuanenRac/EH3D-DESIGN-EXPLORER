# EH3D-DESIGN-EXPLORER: scope

Status: **planned**. EH3D-DESIGN-EXPLORER has no code. This page says what it is meant to do, what it will not do, and where it sits. Nothing here has been built or run.

## Purpose

EH3D-DESIGN-EXPLORER is planned as an environment to compare and analyse design alternatives or branches.

Environment to compare and analyse alternatives or branches of a design by cost, size, losses, temperature and manufacturability.

## What it is planned to do

* Compare alternatives by cost, size, losses, temperature and manufacturability.
* Show the trade-offs instead of a single claimed optimum.
* First version: parametric search and deterministic optimisation. Artificial intelligence only afterwards.

## What it will not do

* It does not claim a single best solution.
* It never applies a change without a reviewable difference.
* It never creates or starts machine jobs.

## Planned shape

A library without a window, written in Rust (planned), with a command-line interface and, where it applies, a kind of task for the server. Programs of the suite exchange work through project files or through the server, never by depending on each other.

Crates, names and file layout will be decided together with the first code; no crate exists yet.

## Where it sits

It belongs to the PHOENIX family of the EH3D Fabrication Suite: independent repositories that relate through versioned contracts. The design tool (EH3D-PHOENIX-PCB) does not depend at build time on the repositories that depend on it.
See DEPENDENCIES.md for the relations, REQUIREMENTS.md for the requirements and STATUS.md for the honest state.
