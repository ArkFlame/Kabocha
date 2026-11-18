# Final rows from docs/tracker/tpar-crates-pumpkin-core-src-entity-mob-warden-rs.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: warden_anger.rs

No generated `pumpkin-data` field is missing and no whole subsystem blocks the row. No STILL OPEN rows remain. No automated test was added: the vibration path has no pure seam (the only deterministic helper, `AngerLevel::by_anger`, is already tested in `warden_anger.rs`), and a formula-mirroring travel-delay test would just duplicate the implementation.

## Result (final pass)

Row `entities/Warden` (the only row in this unit): VERIFIED. No code change was needed in this pass. `warden_anger.rs` already matches the decompiled vanilla anger logic behaviour-for-behaviour, and the parent audit (`docs/tracker/tpar-crates-pumpkin-core-src-entity-mob-warden-rs.md`) confirmed the other three sub-features exist in `warden.rs`. No generated `pumpkin-data` field and no whole subsystem is missing; nothing is BLOCKED-DATA.

- AngerLevel: VERIFIED. Thresholds 0/40/80, ambient/listening sounds, `SORTED_LEVELS` descending sort, `by_anger` and `is_angry` match `.vendor/vanilla-decompiled/net/minecraft/world/entity/monster/warden/AngerLevel.java:8-50`.
- AngerManagement: VERIFIED. Tick conversion delay 2, uuid anger decay, suspect decay/removal with the three unload reasons serialized back to uuid, suspect sort (angry, then player, then higher anger), highest-anger update, `increase_anger` with 150 clamp plus serialized uuid anger, `clear_anger`, `get_active_anger`/`get_active_entity`, and NBT suspects list all match `.vendor/vanilla-decompiled/net/minecraft/world/entity/monster/warden/AngerManagement.java:29-199`.
- Warden anger integration: VERIFIED. `get_anger_level`/`get_active_anger`, `increase_anger_at` (no-AI and can-target guards, maybe-switch-target on player, listening sound), `get_entity_angry_at`, `update_target_from_anger`, and the hurt anger boost (ANGRY minimum + 20) match `.vendor/vanilla-decompiled/net/minecraft/world/entity/monster/warden/Warden.java:432-522`.
- Vibration listening, sniffing and sonic-boom combat: VERIFIED as present by the parent audit against `Warden.java`, `.vendor/vanilla-decompiled/net/minecraft/world/entity/monster/warden/WardenAi.java` and `.vendor/vanilla-decompiled/net/minecraft/world/entity/ai/behavior/warden/SonicBoom.java`. The sniff pose is client-only (no server gameplay effect).

No new automated test: the only pure seam, `AngerLevel::by_anger`, is already covered by `anger_extremes_map_to_calm_and_angry` in `warden_anger.rs`; a decay/cooldown formula test would only mirror the implementation.
