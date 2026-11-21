# EH3D-DESIGN-EXPLORER: requirements

This register is the single source of the state of each capability: `STATUS.md` and the README files cite it and never state a state that it does not. A state is one of `planned`, `prototype`, `implemented` and `validated`. `validated` means that a person has run the capability on a real machine and the evidence of the manifest says `bench` or `field`; nothing here is validated, because nothing of this program has been run with the alternatives of a real design or with the real tools that compute their figures.

The register has 31 requirements: 25 implemented, 4 prototype, 2 planned and 0 validated. By progress: 25 done, 4 partial, 1 pending and 1 blocked. High coverage is not proof of correctness, so each row says how it is checked: the test that names it, and what it shows.

## How to read the register

* **ID:** `DEXP-NNN`, never reused and never renumbered; a new capability takes the next number.
* **State:** `implemented` is code that is complete and tested on a computer, `prototype` is part of the capability, `planned` is not written.
* **Progress:** `done`, `partial`, `pending` (can be written without hardware) or `blocked` (waits for hardware or for another program of the suite).
* **Evidence:** `simulator` when the test runs against a seeded twin of the tools that compute the figures, with failures injected on purpose, `host-test` when it checks a calculation, a reader or a screen-less piece, `none` when nothing is written yet.
* **Test:** the acceptance criterion: `file::name` is the test `name` of `tests/file.rs` (`src/file::name` is a test inside `src/file.rs`), which must exist. For a capability that is not written the cell says `planned:` and the name of the test that will show it; the day that test exists, the row must change.
* **Limits:** what is not done or not proven; a row that is `partial` or `blocked` always says it.

The check is `tests/reqs.rs`: it reads this table, finds each test and fails when a state, a test or a count does not agree.

## Alternatives, sweep and front

| ID | Requirement | State | Progress | Evidence | Test | Limits |
|---|---|---|---|---|---|---|
| DEXP-001 | Define parametric alternatives of a design that can be rebuilt from their parameters. | prototype | partial | host-test | `src/simulation::the_difference_between_two_alternatives_lists_what_the_sweep_changed` | An alternative is a name, its parameters and its figures in a JSON file; the explorer does not compute the figures. |
| DEXP-002 | Deterministic parametric sweep. | prototype | partial | host-test | `src/sweep::the_same_specification_gives_the_same_alternatives` | The sweep is a plain grid in a fixed order with no randomness; searches with seeds are not written. |
| DEXP-003 | Compare across criteria with the source of each figure. | prototype | partial | host-test | `src/explore::check_finds_duplicates_missing_sources_and_bad_numbers` | Every figure carries the tool and version that produced it and a figure without one is flagged; whether the source is real cannot be known. |
| DEXP-004 | Trade-off view: no alternative is hidden by a single score. | prototype | partial | host-test | `src/explore::a_trade_off_keeps_both_and_a_dominated_one_names_who_beats_it` | The non-dominated set is shown with, for every other alternative, who beats it; the criteria and their direction are given by the study. |
| DEXP-005 | Compare branches of the design history. | planned | pending | none | `planned:branches::branches_are_compared_by_their_semantic_differences` | Not written: it needs the history of the design to say what a branch is. |
| DEXP-006 | A figure for a criterion the study does not declare is a finding. | implemented | done | host-test | `src/explore::a_figure_for_a_criterion_the_study_does_not_declare_is_a_finding` | - |
| DEXP-007 | A name that is empty or only blanks is a finding. | implemented | done | host-test | `src/explore::a_name_that_is_empty_or_only_blanks_is_a_finding` | - |
| DEXP-008 | The codes of the findings are all different and follow the numbers. | implemented | done | host-test | `src/explore::the_codes_are_all_different_and_follow_the_numbers` | - |
| DEXP-009 | Equal alternatives do not beat each other. | implemented | done | host-test | `src/explore::equal_alternatives_do_not_beat_each_other` | - |
| DEXP-010 | The direction of a criterion matters. | implemented | done | host-test | `src/explore::the_direction_of_a_criterion_matters` | - |
| DEXP-011 | A tolerance for equality changes what counts as better. | implemented | done | host-test | `src/explore::a_tolerance_for_equality_changes_what_counts_as_better` | - |
| DEXP-012 | An alternative with a figure that is not finite is incomplete and never on the front. | implemented | done | host-test | `src/explore::an_alternative_with_a_metric_that_is_not_finite_is_incomplete_and_never_on_the_front` | - |
| DEXP-013 | An incomplete alternative is not compared and is flagged. | implemented | done | host-test | `src/explore::an_incomplete_alternative_is_not_compared_and_is_flagged` | - |
| DEXP-014 | The difference of two alternatives lists the parameters and the figures with the better side. | implemented | done | host-test | `src/explore::the_difference_lists_parameters_and_metrics_with_the_better_side` | - |
| DEXP-015 | The grid has the product of the values in a fixed order. | implemented | done | host-test | `src/sweep::the_grid_has_the_product_of_the_values_in_a_fixed_order` | - |
| DEXP-016 | A range includes its stop without float drift. | implemented | done | host-test | `src/sweep::a_range_includes_its_stop_without_float_drift` | - |
| DEXP-017 | Bad specifications are refused. | implemented | done | host-test | `src/sweep::bad_specifications_are_refused` | - |
| DEXP-018 | The size of the grid is limited before it is built. | implemented | done | host-test | `src/sweep::the_size_of_the_grid_is_limited_before_it_is_built` | - |

## The command line and the settings

| ID | Requirement | State | Progress | Evidence | Test | Limits |
|---|---|---|---|---|---|---|
| DEXP-019 | `front` shows every alternative and names no winner. | implemented | done | host-test | `src/cli::front_shows_every_alternative_and_names_no_winner` | - |
| DEXP-020 | `check`, `compare` and `sweep` work on files. | implemented | done | host-test | `src/cli::check_compare_and_sweep_work_on_files` | - |
| DEXP-021 | Findings exit with 1 and bad input exits with 2. | implemented | done | host-test | `src/cli::findings_exit_1_and_bad_input_exits_2` | - |
| DEXP-022 | `front` and `compare` run the check and still answer. | implemented | done | host-test | `src/cli::front_and_compare_run_the_check_and_exit_1_with_findings_but_still_answer` | - |
| DEXP-023 | The settings limit the sweep. | implemented | done | host-test | `src/cli::the_settings_limit_the_sweep` | - |
| DEXP-024 | A limit of bytes above zero is kept and a negative or fractional one is refused. | implemented | done | host-test | `src/config::a_limit_of_bytes_above_zero_is_kept_and_a_negative_or_fractional_one_is_refused` | - |

## Simulation of the tools that compute the figures

| ID | Requirement | State | Progress | Evidence | Test | Limits |
|---|---|---|---|---|---|---|
| DEXP-025 | The same seed gives the same study and another seed other figures. | implemented | done | simulator | `src/simulation::the_same_seed_gives_the_same_study_and_another_seed_other_figures` | The twin imitates the form of the figures, not the tools. |
| DEXP-026 | The non-dominated set is consistent and names no single best. | implemented | done | simulator | `src/simulation::the_non_dominated_set_is_consistent_and_names_no_single_best` | - |
| DEXP-027 | Every failure is reported with its code. | implemented | done | simulator | `src/simulation::every_failure_is_reported_with_its_code` | - |
| DEXP-028 | An alternative that crashed is left out of the comparison. | implemented | done | simulator | `src/simulation::an_alternative_that_crashed_is_left_out_of_the_comparison` | - |
| DEXP-029 | Recovery runs the failed alternatives again and the study is clean. | implemented | done | simulator | `src/simulation::recovery_runs_the_failed_alternatives_again_and_the_study_is_clean` | - |
| DEXP-030 | The difference between two alternatives lists what the sweep changed. | implemented | done | simulator | `src/simulation::the_difference_between_two_alternatives_lists_what_the_sweep_changed` | - |
| DEXP-031 | Use the figures of the real tools of the suite. | planned | blocked | none | `planned:real_tools::a_study_with_figures_from_the_real_tools_gives_the_expected_front` | Waits for the tools that compute cost, size, losses and temperature; until then only data made by the tests has been used. |
