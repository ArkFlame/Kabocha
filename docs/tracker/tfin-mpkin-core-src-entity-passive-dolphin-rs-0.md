# Final rows from docs/tracker/tpar-crates-pumpkin-core-src-entity-passive-dolphin-rs.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

matches vanilla, so no source edit and no unit test seam exists. No STILL OPEN rows remain.

## Result

FINAL PASS (terminal). Independently re-verified every row against decompiled vanilla at
.vendor/vanilla-decompiled/net/minecraft/, smallest-first. All three rows are VERIFIED:
current Pumpkin source matches vanilla, so no source edit and no unit test seam exists.

- entities/Dolphin [partial/AUDITED]: VERIFIED.
  .vendor/vanilla-decompiled/net/minecraft/world/entity/animal/dolphin/Dolphin.java:74 is
  "public class Dolphin extends AgeableWaterCreature", with BABY_SCALE (:81),
  getBreedOffspring (:108) and finalizeSpawn using new AgeableMob.AgeableMobGroupData(0.1F)
  (:103). Pumpkin keeps impl AgeableMob (dolphin.rs:148), impl AgeableWaterCreature (:158),
  default_baby_spawn_chance 0.1 (:153), finalize_ageable_spawn (:325) and the ageable tracker
  writes (:331-334). Matches; no change.
- own/tentities-dolphin-54a96a [partial/OWN-OPEN]: VERIFIED (all 4 underlying gaps). Same
  vanilla source confirms: (1) ageable model above; (2) mobInteract gates on ItemTags.FISHES
  (Dolphin.java:323), Pumpkin item.has_tag with tag::Item::MINECRAFT_FISHES (dolphin.rs:414).
  No dolphin_food tag exists in this version, so not data-blocked; (3)
  DolphinJumpGoal.canContinueToUse
  (.vendor/vanilla-decompiled/net/minecraft/world/entity/ai/goal/DolphinJumpGoal.java:56-59) is
  (!(yd*yd<0.03F) || getXRot()==0 || !(abs(getXRot())<10) || !isInWater()) && !onGround(), the OR
  grouping in Pumpkin goals.rs:427-435; (4) registerGoals (Dolphin.java:176-177) registers
  MoveToItemGoal at 7 and PlayWithItemsGoal at 8, matching Pumpkin goals.rs:69-70. No change.
- own/tentities-dolphin [partial/OWN-OPEN]: VERIFIED. The 3-gap audit (tentities-dolphin.md)
  re-confirmed against the same decompiled Dolphin.java and DolphinJumpGoal.java; all NO GAP,
  no source drift. No change.

Files touched: this doc only. rustfmt not applicable (no .rs edit). No unit test added: no code
seam, all rows match decompiled vanilla. No STILL OPEN rows remain.
