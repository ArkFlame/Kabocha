# Parity rows for crates/pumpkin-core/src/entity/mob/illusioner.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **entities/Illusioner** [partial/UNCOVERED] The source has no ranged-bow goal, spell state or spell handlers. Raid movement does not establish illusioner combat. 

## Result

- entities/Illusioner -> FIXED (bow goal, spell state/atomics, spell handlers and raid combat goals were added by earlier passes). This pass verified the full spell/AI flow against vanilla and corrected four parity mismatches:
  - get_celebrate_sound returned EntityEvokerCelebrate instead of EntityIllusionerAmbient. VERIFIED: .vendor/vanilla-decompiled/net/minecraft/world/entity/monster/illager/Illusioner.java getCelebrateSound() returns SoundEvents.ILLUSIONER_AMBIENT (lines 130-133).
  - Blindness spell gated on Normal or Hard difficulty; vanilla gates on isHarderThan(Difficulty.NORMAL.ordinal()) i.e. effectiveDifficulty > 2 => Hard only. Changed to Difficulty::Hard. VERIFIED: same Illusioner.java blindness canUse() line 223; DifficultyInstance.isHarderThan() lines 31-33 and Difficulty ordinals (PEACEFUL=0..HARD=3) in net/minecraft/world/DifficultyInstance.java and net/minecraft/world/Difficulty.java.
  - lastTargetId was recorded at spell-cast completion rather than at goal start; vanilla records it in the blindness goal start(). Moved the assignment into start() and removed the late one. VERIFIED: same Illusioner.java lines 226-233 (IllusionerBlindnessSpellGoal.start).
  - Mirror spell never applied the invisibility effect (only a client-side tick counter). Vanilla performSpellCasting() adds MobEffects.INVISIBILITY for 1200 ticks; now calls living.add_effect(StatusEffect::INVISIBILITY, ILLUSION_LIFETIME=1200). VERIFIED: same file, IllusionerMirrorSpellGoal.performSpellCasting() lines 278-280.

No generated-data, WIT or cross-crate changes required (all referenced Sound/StatusEffect/Effect entries already exist in generated pumpkin-data). Client-side illusion-offset rendering (aiStep/getIllusionOffsets, Illusioner.java lines 92-149) is client-only and intentionally not reproduced on the server, per AGENTS.md. No pure seam exists for a unit test without a full world/mob fixture, and a formula-mirroring test is discouraged, so none added.

No STILL OPEN rows remain.
