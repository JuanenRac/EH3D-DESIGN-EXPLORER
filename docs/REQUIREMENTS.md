# EH3D-DESIGN-EXPLORER: requirements

This register is the single source of the state of each capability. Status values: planned, prototype, implemented, validated. 4 requirements are `prototype` (code and tests exist, only on data made by the tests) and 1 are `planned` (DEXP-005); none has been used on real data or validated. High test coverage is not proof of geometric or electrical correctness, so each requirement says how it will be checked and what result is expected.

| Requirement | Description | Status | How it will be checked | Expected result | Known limits |
|---|---|---|---|---|---|
| DEXP-001 | Define parametric alternatives of a design. | prototype | automatic | Each alternative is reproducible from its parameters. | An alternative is a name, its parameters and its figures in a JSON file, so it can be rebuilt from its parameters; the explorer does not compute the figures. |
| DEXP-002 | Deterministic parametric sweep. | prototype | automatic | Same seeds give the same results. | The sweep is a plain grid in a fixed order with no randomness, so the same specification gives the same alternatives; searches with seeds are not written. |
| DEXP-003 | Compare across criteria with the source of each number. | prototype | automatic | Every figure cites the tool and version that produced it. | Every figure carries the tool and version that produced it and a figure without one is flagged; whether the source is real cannot be known. |
| DEXP-004 | Trade-off view. | prototype | automatic plus review | No alternative is hidden by a single score. | The non-dominated set is shown with, for every other alternative, who beats it; an alternative that lacks a figure is marked incomplete and hides no other. |
| DEXP-005 | Compare branches of the design history. | planned | automatic | Branches are compared by their semantic differences. | Not started. |

Rules: a claim of correctness names the corpus, the tolerance and the tool version it holds for. A phase is complete when the measured rate does not drop, every failure is classified and fixed or documented, and the measured figure is reported.
