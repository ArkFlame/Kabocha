# Final rows from docs/tracker/tentities-dark-oak-boat.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: boat.rs

- Row 3 (BUBBLE_TIME): verified already fixed in the current `boat.rs`. `tick_underwater` (lines 392-419) now only increments `ticks_underwater` for the 60-tick eject and deliberately does not write `ID_BUBBLE_TIME` (comment lines 393-395). A grep for `ID_BUBBLE_TIME`/`bubble_time` in pumpkin-core finds exactly three writes, all in bubble-column paths: `on_above_bubble_column` (639-642, sets 60 once) and `tick_bubble_column` (436, 446, decrement to 0 and clear). This matches vanilla `AbstractBoat.onAboveBubbleColumn` (line 165-172, `setBubbleTime(60)` only when 0) and `tickBubbleColumn` (351-387, decrement while above a column, then sink/launch effect at 0); `handleBubbleColumnEffect` (399-408) is mirrored by `bubble_column_vertical_velocity`. No code change. The prior STILL OPEN note and its boat.rs:286-291 line reference are stale.
Third re-review pass against the decompiled tree at `.vendor/vanilla-decompiled` (AbstractBoat, BoatItem, EntitySelector, Entity, LivingEntity, Player). Only the prior STILL OPEN items were revisited.
Third-pass final review against `.vendor/vanilla-decompiled/net/minecraft/world/entity/vehicle/boat/AbstractBoat.java` (plus BoatItem / EntitySelector / Player / BoatDispenseItemBehavior). Every row is now terminal; no STILL OPEN rows remain.
Fourth-pass verification. Read the unit's last Result and the shipped source; every row is terminal and no STILL OPEN row remains.

## Result
STILL OPEN until closed. Per item record FIXED, VERIFIED (vanilla path) or BLOCKED-DATA (exact missing generated field).
