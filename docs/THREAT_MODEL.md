# EH3D-DESIGN-EXPLORER: threat model

Basic model for early code. The rows marked partial have a mitigation written and tests for it, run only on data made by the tests themselves; the rows marked planned have nothing yet. No row is assessed against real data.

| Threat | Planned mitigation | Planned test | State | Residual risk |
|---|---|---|---|---|
| Resource exhaustion by a huge parameter space | the settings limit the evaluations and the size of the grid, and the grid is refused before it is built | `the_size_of_the_grid_is_limited_before_it_is_built`, `the_settings_limit_the_sweep` | partial | time and memory of one evaluation are not limited; no fuzzing |
| A misleading comparison built on inconsistent tool versions | every figure must name the tool and the version that produced it, and the check flags one that does not | `check_finds_duplicates_missing_sources_and_bad_numbers` | partial | the versions are recorded and flagged when missing; they are not compared across alternatives |
| Corrupt or malicious project or input files (oversized entries, path tricks, deep nesting, resource exhaustion) | the JSON reader is strict (unknown fields refused) and limited in size | `json_is_strict_and_limited` | partial | hostile samples written by hand; no fuzzing yet |
| Temporary files, logs and reports that reveal private designs | no private data in outputs; redaction | automated pattern search | planned | not assessed: nothing is implemented |
| Compromised dependency or binary | pinned versions, licence and advisory checks, software bill of materials for releases | CI checks | planned | not assessed: nothing is implemented |
| Power loss or forced close during a save | atomic write: temporary file, then rename; old file never deleted first | interruption tests at controlled points | planned | not assessed: nothing is implemented |

Minimum negative tests once there is code: hostile files and resource limits; secrets absent from logs, errors and reports; pinned dependencies and licence review before publishing. Report a vulnerability privately as described in SECURITY.md, without attaching private designs.
