# Final rows from docs/tracker/tpar-crates-pumpkin-core-src-entity-player-advancement-trigger-rs.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files:

The single inherited row is `own/tentities-fishing-bobber-e0f308`, an "own audit" summary pointing to
`docs/tracker/tentities-fishing-bobber-e0f308.md` (whose sub-rows were closed in earlier passes). Its
only sub-row touching this unit's file (`trigger.rs`, the full-list `FISHING_ROD_HOOKED` trigger) is
closed below. No source change was needed this pass: the code already matches decompiled vanilla.

## Result

- `own/tentities-fishing-bobber-e0f308` (sub-row 8, single full-list `FISHING_ROD_HOOKED` trigger):
  VERIFIED FIXED. Vanilla
  `.vendor/vanilla-decompiled/net/minecraft/advancements/triggers/FishingRodHookedTrigger.java:56-85`
  (`matches`: the `item` predicate passes when ANY caught ItemStack satisfies it, lines 72-77) and
  `.vendor/vanilla-decompiled/net/minecraft/world/entity/projectile/FishingHook.java:524,537`
  (entity path fires once with `Collections.emptyList()`; item path fires once with the full list).
  Pumpkin: `AdvancementTrigger::FishedItem { items: Vec<(String, u32)> }` (trigger.rs:46-48) with
  matcher `items.iter().any(|(item_id, count)| pred.matches(item_id, *count, &[]))`
  (trigger.rs:451-461). Both call sites fire once: empty vec on the entity path
  (`fishing_bobber.rs:219-223`) and the full caught list before the spawn loop
  (`fishing_bobber.rs:263-267`). All repo-wide `FishedItem` uses carry the new shape, so types line up.
  Matches vanilla, no source change needed.
  No test added: the matcher needs a live `Player` with advancement listeners (no pure seam) and the
  `.any(...)` rule is trivial.
