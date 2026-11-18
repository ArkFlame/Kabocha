# Parity rows for crates/pumpkin-core/src/command/commands/give.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **own/tentities-item-0024a5** [partial/OWN-OPEN] own audit: 10 open, 1 fixed, 0 verified, 0 blocked-data (docs/tracker/tentities-item-0024a5.md) docs/tracker/tentities-item-0024a5.md

## Result
own/tentities-item-0024a5: VERIFIED. The referenced item tracker
(docs/tracker/tentities-item-0024a5.md) has no STILL OPEN rows remaining; its final
verification pass closed all 10 rows. Re-checked give.rs against
.vendor/vanilla-decompiled/net/minecraft/server/commands/GiveCommand.java:
- MAX_ALLOWED_ITEMSTACKS = 100 and the max_stack * 100 cap / TOO_MANY_ITEMS error
  (give.rs:34,61-69) match vanilla lines 33,64-66.
- while remaining > 0 with take = min(max_stack, remaining) and the two branches
  (give.rs:74-108) match vanilla lines 77-109: the all-fits branch spawns a
  make_fake_item() entity, plays EntityItemPickup at volume 0.2 with pitch
  (rand - rand) * 0.7 + 1.0, times 2.0, and syncs inventory (vanilla 82-100); the
  did-not-fit branch calls set_pickup_delay(0) then set_target (vanilla 101-107).
- single/multiple feedback (give.rs:111-131) matches vanilla lines 25-32.
No give.rs source change was needed this pass; the code already matches vanilla. No
pure unit test seam exists for the fake-item branch (it mutates atomics on a live
ItemEntity needing Arc<World>). No rustfmt run (no .rs edit). Re-read the diff against
the worker compile pitfalls: no new imports, helpers, macro/struct-init/clone changes,
unwrap/expect/panic or em dashes.
