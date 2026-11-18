# Parity rows for crates/pumpkin-core/src/entity/mob/ravager.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **entities/Ravager** [partial/COVERED] The source registers normal raid combat but has no stun/roar state or handlers. 

## Result
- **entities/Ravager** FIXED / VERIFIED against `.vendor/vanilla-decompiled/net/minecraft/world/entity/monster/Ravager.java`. Added the three vanilla tick counters (`attackTick`, `stunnedTick`, `roarTick`) with NBT persistence (`AttackTick`/`StunTick`/`RoarTick`, Java lines 109-123); `aiStep` timer progression with roar at `roarTick==10` and stun-finish roar sound + `roarTick=20` (lines 136-193); `isImmobile` (205-208); `hasLineOfSight` blocked while stunned/roaring (210-213); `blockedByItem` shield-block 50% stun with sound/status event and strong knockback (215-229); `doHurtTarget` attack animation event + sound (301-307); and `roar()` AoE 6.0 MOB_ATTACK damage, knockback, illager/self/ griefing exclusions and RAVAGER_ROARED status (231-248). Added two pure timer state-machine unit tests. Skipped client-only effects (stun/roar particles and GameEvent) per the client-only rule.

