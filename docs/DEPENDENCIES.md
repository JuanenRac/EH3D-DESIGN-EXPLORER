# EH3D-DESIGN-EXPLORER: dependencies

One row per relation with another repository. The kind is conceptual, compile-time or run-time. The state of a contract is: not written, drafted, accepted, frozen or published. A repository existing does not mean that its contract is ready: this project does not implement or consume a contract that is not accepted.

| From | To | Kind | Contract | Contract state | Minimum version | Tested version | Compatibility test | Change owner | What it blocks | Migration |
|---|---|---|---|---|---|---|---|---|---|---|
| EH3D-DESIGN-EXPLORER | EH3D-PHOENIX-PCB | compile | design model | drafted | not set | not set | to be written with the first code | the owner of the contract | all of it | to be defined |
| EH3D-DESIGN-EXPLORER | EH3D-DESIGN-HISTORY | conceptual | branches to compare | not written | not set | not set | to be written with the first code | the owner of the contract | branch comparison | to be defined |
| EH3D-DESIGN-EXPLORER | EH3D-DESIGN-VERIFIER | conceptual | manufacturability and checks | not written | not set | not set | to be written with the first code | the owner of the contract | the criteria | to be defined |
| EH3D-DESIGN-EXPLORER | EH3D-THERMAL-LAB | conceptual | temperature figures | not written | not set | not set | to be written with the first code | the owner of the contract | the criteria | to be defined |
| EH3D-DESIGN-EXPLORER | EH3D-MANUFACTURING-PROFILE | conceptual | fabrication limits | not written | not set | not set | to be written with the first code | the owner of the contract | the criteria | to be defined |

Rules: repositories are built next to each other as sibling folders and each declares in its manifest the versions of the others that it needs, once those exist. A lower layer never depends on a higher one. The manifest field for build dependencies stays empty until code exists.
