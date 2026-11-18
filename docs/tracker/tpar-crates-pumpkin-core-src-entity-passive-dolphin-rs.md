# Parity rows for crates/pumpkin-core/src/entity/passive/dolphin.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **entities/Dolphin** [partial/AUDITED] The reviewed source has changed. The previous assessment needs a new review. The reviewed source has changed.
- **own/tentities-dolphin-54a96a** [partial/OWN-OPEN] own audit: 4 open, 0 fixed, 0 verified, 1 blocked-data (docs/tracker/tentities-dolphin-54a96a.md) docs/tracker/tentities-dolphin-54a96a.md
- **own/tentities-dolphin** [partial/OWN-OPEN] own audit: 1 open, 0 fixed, 0 verified, 1 blocked-data (docs/tracker/tentities-dolphin.md) docs/tracker/tentities-dolphin.md

## Result

THIRD PASS (terminal). All three rows verified against decompiled vanilla at
.vendor/vanilla-decompiled/net/minecraft/. Every row is VERIFIED: current Pumpkin source
matches vanilla, so no source edit and no unit test seam exists. No STILL OPEN rows remain.

- entities/Dolphin [partial/AUDITED]: VERIFIED.
  .vendor/vanilla-decompiled/net/minecraft/world/entity/animal/dolphin/Dolphin.java:74 is
  "public class Dolphin extends AgeableWaterCreature", with BABY_SCALE (:81),
  getBreedOffspring (:108) and finalizeSpawn using new AgeableMob.AgeableMobGroupData(0.1F)
  (:103). Pumpkin keeps impl AgeableMob (dolphin.rs:148), impl AgeableWaterCreature (:158),
  default_baby_spawn_chance 0.1 (:153), finalize_ageable_spawn (:325) and the ageable tracker
  writes (:331-334). Matches; no change needed.
- own/tentities-dolphin-54a96a [partial/OWN-OPEN]: VERIFIED (all 4 underlying gaps). Same
  vanilla source confirms: (1) ageable model above; (2) mobInteract gates on
  ItemTags.FISHES (Dolphin.java:323), Pumpkin item.has_tag with tag::Item::MINECRAFT_FISHES
  (dolphin.rs:414). No dolphin_food tag exists in this version, so not data-blocked;
  (3) DolphinJumpGoal.canContinueToUse
  (.vendor/vanilla-decompiled/net/minecraft/world/entity/ai/goal/DolphinJumpGoal.java:58-59) is
  (!(yd*yd<0.03F) || getXRot()==0 || !(abs(getXRot())<10) || !isInWater()) && !onGround(), the OR
  grouping in Pumpkin goals.rs:427-435; (4) registerGoals (Dolphin.java:176-177) registers
  MoveToItemGoal at 7 and PlayWithItemsGoal at 8, matching Pumpkin goals.rs:69-70. No change.
- own/tentities-dolphin [partial/OWN-OPEN]: VERIFIED. The 3-gap audit (tentities-dolphin.md)
  re-confirmed against the same decompiled Dolphin.java and DolphinJumpGoal.java; all NO GAP,
  no source drift. No change needed.

Files touched: this doc only. rustfmt not applicable (no .rs edit). No unit test added: no code
seam, all rows match decompiled vanilla.
