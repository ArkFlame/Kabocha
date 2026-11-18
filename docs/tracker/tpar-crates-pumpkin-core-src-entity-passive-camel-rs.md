# Parity rows for crates/pumpkin-core/src/entity/passive/camel.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **entities/Camel Husk** [partial/AUDITED] The reviewed source has changed. The previous assessment needs a new review. The reviewed source has changed.

## Result

- entities/Camel Husk: FIXED. Vanilla CamelHusk.interact (.vendor/vanilla-decompiled/net/minecraft/world/entity/animal/camel/CamelHusk.java:40-43) calls setPersistenceRequired() before delegating to super.interact. Pumpkin only did this in mob_interact_with_hand (camel.rs:477-489), which is the Bedrock-only interact_with_hand path; the Java interact path (net/java/play/interact.rs:147 -> entity/mod.rs:1080 Mob::interact -> mob/mod.rs:3296 -> mob_interact) never set it. Added the husk persistence store at the top of CamelEntity::mob_interact (camel.rs:618-624), mirroring the vanilla override for Java interactions; the Bedrock path still sets it too.

Verified remaining CamelHusk overrides against the same vanilla file (all already match, no edit needed): removeWhenFarAway -> true (camel.rs remove_when_far_away returns is_husk()); isMobControlled -> first passenger is a Mob (is_mob_controlled); canBeLeashed -> !isMobControlled() (can_be_leashed); isFood -> CAMEL_HUSK_FOOD tag (is_food); canMate/getBreedOffspring/canBeABaby -> no breeding (can_mate, can_be_a_baby); canFallInLove -> false (can_fall_in_love, love gate in handle_eating); chargeSpeedModifier -> 4.0 (charge_speed_modifier); step sounds at 0.4 volume (mob_play_step_sound). Sound overrides (getAmbientSound, getDeathSound, getHurtSound, getDashingSound, getDashReadySound, getEatingSound, getStandUpSound, getSitDownSound, getSaddleSound) are all sourced from generated EntityType::CAMEL_HUSK fields (entity_type.rs:1854-1865, generated data present, no BLOCKED-DATA).

No new unit test: constructing a CamelEntity needs Entity/MobEntity/World, so there is no pure seam; a getter test would be a disallowed getter test. UNVERIFIED (no cargo per worker rules; orchestrator compiles).
