---
name: kabocha-mvp-tests
description: Decide what tests to write or delete: minimal automated tests that catch real regressions.
---
# MVP tests
Write: codec/parser round trips vs known vanilla bytes; ordering/boundary logic; a bug repro that FAILS without the fix (verify by stash). Gameplay needing a world: pumpkin-gametest.
Never: tests mirroring the implementation formula, constants/generated-data checks, getters/Default/derive, duplicates, over-mocked tests.
One focused test per behaviour, beside its neighbours. When triaging a file, delete tests matching the Never list. Every behaviour change ships its smallest automated test; tests are the gate. Gate: `scripts/check.sh -f <crate>`; `scripts/boot.sh` only for runtime changes.
