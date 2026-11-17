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
