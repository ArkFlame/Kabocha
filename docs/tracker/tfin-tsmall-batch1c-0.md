# Final rows from docs/tracker/tsmall-batch1c.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: axolotl.rs

5. `taudit-crates-pumpkin-core-src-en-f50f1a.md:14` axolotl play-dead causing entity: STILL OPEN (divergence found). Axolotl.java:397-405 passes when `source.getEntity() != null || source.getDirectEntity() != null`. Rust `axolotl.rs` `HurtCheck.has_causing_entity` uses only `cause.is_some()`, so a direct entity without an owner does not count. Fix needs the direct source threaded into `on_damage_with_context`; not a one-line change, so left open.
6. `taudit-crates-pumpkin-core-src-en-f50f1a.md:15` axolotl play-dead health before hit: STILL OPEN (divergence found). Vanilla reads `currentHealth` before `super.hurtServer`. Rust `on_damage_with_context` runs after damage is applied and reads `living.health`, so the health ratio and `damage < health` use post-hit health. Its doc comment claims the opposite. Fix needs a pre-damage health snapshot; left open.

## Result
STILL OPEN until closed. Per item record FIXED, VERIFIED (vanilla path) or BLOCKED-DATA (exact missing generated field).
