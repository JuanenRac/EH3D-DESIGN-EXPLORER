# EH3D-DESIGN-EXPLORER: requirements

This register is the single source of the state of each capability. Status values: planned, prototype, implemented, validated. Every requirement is `planned` today: nothing is implemented or verified. High test coverage is not proof of geometric or electrical correctness, so each requirement says how it will be checked and what result is expected.

| Requirement | Description | Status | How it will be checked | Expected result | Known limits |
|---|---|---|---|---|---|
| DEXP-001 | Define parametric alternatives of a design. | planned | automatic | Each alternative is reproducible from its parameters. | Not started: no limits known yet. |
| DEXP-002 | Deterministic parametric sweep. | planned | automatic | Same seeds give the same results. | Not started: no limits known yet. |
| DEXP-003 | Compare across criteria with the source of each number. | planned | automatic | Every figure cites the tool and version that produced it. | Not started: no limits known yet. |
| DEXP-004 | Trade-off view. | planned | automatic plus review | No alternative is hidden by a single score. | Not started: no limits known yet. |
| DEXP-005 | Compare branches of the design history. | planned | automatic | Branches are compared by their semantic differences. | Not started: no limits known yet. |

Rules: a claim of correctness names the corpus, the tolerance and the tool version it holds for. A phase is complete when the measured rate does not drop, every failure is classified and fixed or documented, and the measured figure is reported.
