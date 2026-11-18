# entity / Parrot — deep audit

Sources: `crates/pumpkin-core/src/entity/passive/parrot.rs` (plus `parrot/shoulder.rs`, `entity/player/shoulders.rs`).
Compared against vanilla `Parrot` / `TamableAnimal` (no decompiled reference in `.vendor/`); behaviour was last landed in 593aac607.

## Markers
0 TODO/FIXME. The tracker's 5 "vanilla" matches are reference comments on already-correct code: `parrot.rs:30,169,186,347,431,519,527,564`.

## Verified matching vanilla
- Goal table: panic 1.25, float, look-at-player 8, sit, follow-owner 1.0/5/1.0, water-avoiding random fly, land-on-shoulder, follow-mob 1.0/3/7 (`parrot.rs:139-156`).
- Interaction order: tame with `parrot_food` (consume, 1/10, entity event), poison only for `parrot_poisonous_food`, sit toggle only when on ground and the held item is not `parrot_food`, then `super.mobInteract` (`parrot.rs:384-428`).
- Cookie: consume, poison 900 ticks, `f32::MAX` player attack with creative bypass (`parrot.rs:171-197`).
- Ambient imitation 1/1000 and non-peaceful check, nearby imitation 1/400 then 1/2, 20-block box, 0.7 volume, random pitch (`parrot.rs:49-103,343-346`).
- `calculateFlapping`, `flap()` (fly sound 0.15/1.0), step sound 0.15/1.0, death/hurt sounds (`parrot.rs:343-378,227-234`).
- Variant: clamp on legacy integer read, random on spawn, tracked data, `ParrotVariant` component read/write (`parrot.rs:280-323`).
- Non-breeding, non-food animal, no fall damage, pushable, persists when far away.

## Remaining gaps
None confirmed. Two items need a decompiled source or runtime check before any change:

| # | Location | Question | Size |
|---|---|---|---|
| 1 | `parrot.rs:452` | `HAPPY_GHAST` maps to `Sound::IntentionallyEmpty` and `imitate_nearby` plays it, returning true. Vanilla maps happy ghast to `SoundEvents.EMPTY`; confirm whether vanilla's imitate path skips EMPTY before deciding to filter. | S |
| 2 | `parrot.rs:343-363` | Flap state and the `velocity.y * 0.6` descent damping are computed in the server AI step; vanilla `Parrot.calculateFlapping` runs in the client-side `aiStep` only. Pumpkin exposes flap state through `mob_is_flapping`/`mob_on_flap` and uses the same pattern for chicken, so left unchanged. Needs runtime observation of parrot fall speed. | M |

## Gap status
- 1 HAPPY_GHAST EMPTY sound (parrot.rs:452): STILL OPEN, needs decompiled imitate path.
- 2 server-side flap/descent damping (parrot.rs:343-363): STILL OPEN, needs runtime observation.

## Result

Decompiled vanilla is available: .vendor/vanilla-decompiled/net/minecraft/world/entity/animal/parrot/Parrot.java
(plus ShoulderRidingEntity.java, TamableAnimal.java and server/level/ServerPlayer.java).

- Row 1 (imitate EMPTY, parrot.rs:452): VERIFIED MATCHING, no change. Vanilla
  imitateNearbyMobs (Parrot.java:234-249) builds candidates with NOT_PARROT_PREDICATE =
  MOB_SOUND_MAP.containsKey, which INCLUDES HAPPY_GHAST -> SoundEvents.EMPTY. It plays
  getImitatedSound(type) (EMPTY for happy ghast) at 0.7 volume and returns true, with no
  skip/filter of EMPTY. Pumpkin imitate_nearby does the same. Left unchanged.
- Row 2 (server-side flap/descent, parrot.rs:343-363): VERIFIED MATCHING, no change.
  Vanilla calculateFlapping (Parrot.java:216-232) runs from aiStep() with NO isClientSide
  guard, so it runs server-side too. Its flapSpeed += (onGround||passenger ? -1 : 4) * 0.3,
  clamp 0..1, and !onGround && movement.y<0 -> multiply(1.0,0.6,1.0) all match Pumpkin
  mob_ai_step exactly. Flap-phase fields (flap/flapping/oFlap) are render-only; Pumpkin
  exposes the equivalent via mob_is_flapping/mob_on_flap. Left unchanged.
- Bonus parity fix (live code): vanilla Parrot.getAmbient (Parrot.java:333-340) picks a
  random key from the WHOLE MOB_SOUND_MAP.keySet(), including HAPPY_GHAST -> EMPTY, so the
  shoulder-parrot ambient (ServerPlayer.java:1202-1221 -> Pumpkin shoulders.rs:127) can
  occasionally be silent. Pumpkin ambient_candidates wrongly filtered HAPPY_GHAST out.
  Removed the filter so ambient() can return Sound::IntentionallyEmpty, matching vanilla.
  Updated the unit test (renamed to ambient_imitation_includes_the_happy_ghast_silent_entry):
  it now asserts all IMITATIONS are candidates and IntentionallyEmpty is present.
- Files touched: crates/pumpkin-core/src/entity/passive/parrot.rs (rustfmt --edition 2024 OK).
  No generated data, WIT, or cross-crate changes needed.

Status: PASS fixed=1 (ambient candidates include the happy ghast silent entry + unit test) verified-no-change=2 (imitate EMPTY, server-side flap/descent) open=0 files=crates/pumpkin-core/src/entity/passive/parrot.rs

## Result (batch r2_008, code re-check)

- Row 1 (`parrot.rs:452` HAPPY_GHAST `IntentionallyEmpty`): VERIFIED. Vanilla `Parrot.imitateNearbyMobs` (`.vendor/vanilla-decompiled/net/minecraft/world/entity/animal/parrot/Parrot.java:240-243`) plays `getImitatedSound(type)` and returns true; the happy ghast entry maps to `SoundEvents.EMPTY` (Parrot.java:99). Rust `crates/pumpkin-core/src/entity/passive/parrot.rs:472` maps it to `Sound::IntentionallyEmpty` and `imitate_nearby` returns true (parrot.rs:84-), same observable result (no audible sound, returns true). No filter added.
- Row 2 (server-side flap/descent damping, `parrot.rs:343-363`): STILL OPEN. Needs runtime observation of parrot fall speed; not reached in this pass.
- Note (not a row, not changed): Rust `imitate_nearby` (parrot.rs:61-72) restricts candidates to the `IMITATIONS` table, while vanilla picks from every non-parrot `Mob` in the box (Parrot.java:237) and only then resolves the sound. Candidate set and random index may differ; needs a separate row.
- Commands: none run (workers do not build). Files edited: none.

## Result (batch r3_027, re-derived from Gap table)

- Row 1 (`parrot.rs:452` HAPPY_GHAST `IntentionallyEmpty`): VERIFIED. Vanilla `Parrot.imitateNearbyMobs` (`.vendor/vanilla-decompiled/net/minecraft/world/entity/animal/parrot/Parrot.java:234-243`) maps happy ghast to `SoundEvents.EMPTY` (Parrot.java:99), plays it and returns true with no filter. Rust `crates/pumpkin-core/src/entity/passive/parrot.rs:472` maps it to `Sound::IntentionallyEmpty` and `imitate_nearby` returns true. No change.
- Row 2 (server-side flap/descent damping, `parrot.rs:343-363`, now `mob_ai_step` at `parrot.rs:361-378`): VERIFIED. Vanilla `calculateFlapping` (Parrot.java:216-232) is called from `aiStep` (Parrot.java:192-204) with no `isClientSide` guard, so it runs on the server. Flap change `(!onGround && !isPassenger ? 4 : -1) * 0.3`, clamp 0..1, and `!onGround && movement.y < 0 -> multiply(1.0, 0.6, 1.0)` match Rust `parrot.rs:366-377` (`on_ground || has_vehicle` is the same predicate as `!onGround && !isPassenger`). Only the flap-phase fields are render-only and are covered by `mob_is_flapping`/`mob_on_flap`. No change.
- Not a row, not changed: the candidate-set difference in `imitate_nearby` (Rust restricts to `IMITATIONS`; vanilla picks any non-parrot `Mob` in the box, Parrot.java:237) is still open and needs its own row.
- Commands: none run (workers do not build). Files edited: none.

Status: PASS verified=2 (happy ghast EMPTY imitate, server-side flap/descent) fixed=0 blocked=0 open=0 files=none
