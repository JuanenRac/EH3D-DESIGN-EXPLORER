# EH3D-DESIGN-EXPLORER: scope

Status: **early code**. EH3D-DESIGN-EXPLORER is a library and a command line that read a set of alternatives of a design, check it, show the place of every alternative without naming a single best one, list what differs between two alternatives and generate a parametric sweep. It has been tried on data made by its own tests only. This page says what it is for, what it will not do and where it sits among the other programs of the suite.

## Purpose

The environment to compare and analyse the alternatives of a design by cost, size, losses, temperature and manufacturability: it shows the trade-offs instead of a single claimed optimum, and every figure it shows names the tool and the version that produced it.

## What it does

* Reads a study (alternatives with their parameters and figures, and the criteria with the direction of each) from a strict JSON file within a size limit.
* Checks it: a figure for a criterion the study does not declare, an empty name, a duplicate, a figure without the tool and version that produced it and a number that is not finite are findings.
* Shows the non-dominated set (`front`) with, for every other alternative, who beats it; an alternative that lacks a figure is marked incomplete, left out of the comparison and hides no other.
* Lists what differs between two alternatives, parameters and figures, with the better side (`compare`).
* Generates the grid of a parametric sweep in a fixed order with no randomness, with the size of the grid limited before it is built (`sweep`).
* Runs a seeded twin of the tools that compute the figures, with three failures injected and a recovery that runs the failed alternatives again.

## What it will not do

* It does not claim a single best solution.
* It never applies a change without a reviewable difference.
* It never creates or starts a machine job.
* It does not compute the figures: they come from other tools, and it compares branches of the design history in no way yet (see `REQUIREMENTS.md`).
* It does not claim that anything was proven with a real design: the evidence of every capability is `simulator` or `host-test`.

## Where it sits

* **It reads:** a study (JSON), a sweep specification (JSON) and a settings file (the size limits, the equality tolerance and the size of the grid).
* **It writes:** the front, the comparison and the grid it is asked to print, to the standard output; it never writes to the files it reads.
* **It is read by:** nothing reads it yet: the clients will show its front once the design tools give figures.
* **It depends on:** nothing of the suite at build time; the design model comes from EH3D-PHOENIX-PCB and the checks from EH3D-DESIGN-VERIFIER.

## What it never decides

Which alternative to build, and the limits, the interlocks and the emergency stop of any machine, belong to the person who decides. The front shows what no other alternative beats on the criteria that were declared; it does not weigh them. Nothing here starts motion.

The requirements of the program, with the state of each and the test that shows it, are in `REQUIREMENTS.md`; the state in a paragraph is in `STATUS.md`.
