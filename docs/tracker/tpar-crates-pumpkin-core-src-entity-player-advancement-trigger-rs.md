# Parity rows for crates/pumpkin-core/src/entity/player/advancement/trigger.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **own/tentities-fishing-bobber-e0f308** [partial/OWN-OPEN] own audit: 19 open, 1 fixed, 1 verified, 0 blocked-data (docs/tracker/tentities-fishing-bobber-e0f308.md) docs/tracker/tentities-fishing-bobber-e0f308.md

## Result
STILL OPEN until closed. Per row record: FIXED, VERIFIED with the vanilla path, or BLOCKED-DATA with the exact missing generated field)


## Result (third pass - all rows closed)

The single row `own/tentities-fishing-bobber-e0f308` is an "own audit" summary pointing to
`docs/tracker/tentities-fishing-bobber-e0f308.md`, whose final Result (second pass) closed every
sub-row. This pass re-verified the one sub-row touching this unit's file (`trigger.rs`) against
decompiled vanilla. `.vendor/vanilla-decompiled/net/minecraft/` does not contain these classes;
the reference used is `/run/media/linsaftw/SDD/tmp/mc-proof/ref/net/minecraft/`.

- Row 8 (single full-list `FISHING_ROD_HOOKED` trigger): VERIFIED FIXED.
  `AdvancementTrigger::FishedItem` carries `items: Vec<(String, u32)>` (trigger.rs:46-48) and the
  matcher is `items.iter().any(|(item_id, count)| pred.matches(item_id, *count, &[]))`
  (trigger.rs:451-461). This matches
  `.../advancements/triggers/FishingRodHookedTrigger.java:56-85`, where the item predicate passes
  when ANY caught item satisfies it (lines 72-77). Both call sites in `fishing_bobber.rs` are
  correct: the entity path fires once with an empty vec (vanilla
  `.../world/entity/projectile/FishingHook.java:524` `Collections.emptyList()`), and the item path
  fires once with the full caught list before the spawn loop (vanilla `retrieve:537`). Types line
  up: `u32::from(stack.item_count)` is passed to `ItemPredicate::matches(&str, u32, &[(..)])`
  (predicate.rs:231). All four repo-wide `FishedItem` usages are the new shape.

No source change needed this pass; the code is already the verified vanilla-matching form. No test
added: the `FishedItem` matcher needs a live `Player` with advancement listeners (no pure seam),
and the `.any(...)` rule is trivial.
