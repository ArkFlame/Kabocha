# Final rows from docs/tracker/tentities-creeper.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

Observation (out of scope, not changed): the same decompiled class gates charged-creeper heads behind `this.droppedSkulls` (one head per explosion), while the row 2 change on master removed that gate. Flagging only; row 2 is outside the STILL OPEN set for this unit.
Re-verified every STILL OPEN row against `.vendor/vanilla-decompiled/net/minecraft/world/entity/monster/Creeper.java` (Paper 1.21.x) and the current Pumpkin sources. No `.rs` edit was needed, so no rustfmt was run.
Second pass (expanded file permissions). Re-verified each STILL OPEN row against `.vendor/vanilla-decompiled/net/minecraft/world/entity/monster/Creeper.java` and `.../ai/goal/SwellGoal.java` (Paper 1.21.x) and the current Pumpkin sources. No `.rs` edit was required.
Third pass (expanded permissions). Re-verified each formerly STILL OPEN row against decompiled vanilla at `.vendor/vanilla-decompiled/net/minecraft/world/entity/monster/Creeper.java` and `.vendor/vanilla-decompiled/net/minecraft/world/entity/AreaEffectCloud.java` (Paper 1.21.x), plus the current Pumpkin sources. No source edit was required for any row, so no rustfmt was run. No row remains STILL OPEN.

## Result
STILL OPEN until closed. Per item record FIXED, VERIFIED (vanilla path) or BLOCKED-DATA (exact missing generated field).
