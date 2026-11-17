# Mob verification against decompiled vanilla, pass 5 (f3)
Source: `.vendor/vanilla-decompiled/net/minecraft/world/entity/...` (read-only). Continues `tverify-mobs3.md` (x10). Base: master. Not compiled: workers do not build. Edits were run through `rustfmt --edition 2024`.

| Item | Path | Status | Evidence |
|---|---|---|---|
| Iron golem `can_attack` GHAST exclusion | entity/passive/iron_golem.rs | VERIFIED: vanilla `IronGolem.canAttack` calls `super.canAttack`, which is `Mob.canAttack` and excludes `GHAST`. Rust excludes ghast explicitly, same result. | Mob.java line 337-339, IronGolem.java line 138-140. |
| Iron golem attack damage and knockback | entity/passive/iron_golem.rs | VERIFIED: `a/2 + nextInt(a)` when `(int)a > 0`, knockback up 0.4 scaled by `max(0, 1 - KB resist)`, attack sound. | IronGolem.java lines 182-201. |
| Llama breed partner filter | entity/passive/llama.rs | VERIFIED: vanilla `BreedGoal` searches by `animal.getClass()`. Non-trader llamas search `LLAMA_LIKE` (includes trader llamas, as `instanceof Llama`). Trader llamas search only trader llamas. Rust matches both. | BreedGoal.java lines 20, 62-64; Llama.java `canMate` line 316-318. |
| Llama spit | entity/passive/llama.rs | PARTIAL: aim `dy` at `height*1/3`, `yo = dist*0.2`, speed 1.5 and `didSpit` revenge handling match. Uncertainty is hardcoded 10.0, but vanilla uses `rangedAttackUncertainty(level)` from canvas config (`base - difficulty*4`). The config base was not located, so not changed. | Llama.java lines 342-364; RangedAttackMob.java line 9-11. |
| Llama caravan goal | entity/ai/goal/llama_follow_caravan.rs | VERIFIED: candidate search (9,4,9), in-caravan then leashed passes, tie-to-later, 4.0 distance gate, `firstIsLeashed` depth 8, 676 range speed bump, stop reset 2.1, tick steering `dist-2`. | LlamaFollowCaravanGoal.java lines 20-126. |
| Llama goals | entity/passive/llama.rs | VERIFIED: caravan 2 (2.1), ranged 3 (1.25, 40, 20), panic 3 (1.2), breed 4, tempt 5, follow 6, stroll 7 (0.7), look 8, random look 9. Revenge with `didSpit` clear. | Llama.java lines 119-135, 461-495. |
| Donkey goals | entity/passive/donkey.rs | VERIFIED: Donkey has no `addGoal`, so it inherits the AbstractHorse list. | Donkey.java (no `registerGoals`). |
| Donkey mate and offspring | entity/passive/donkey.rs | VERIFIED: partners Donkey or Horse, offspring Mule when partner is Horse, else Donkey. | Donkey.java lines 46-66. |
| Mule goals and mate | entity/passive/mule.rs | VERIFIED: Mule has no `addGoal` and no `canMate`, so `AbstractHorse.canMate` returns false. Rust returns false. | AbstractHorse.java line 822-824; Mule.java. |
| Abstract horse goals (recheck) | entity/passive/abstract_horse.rs | CORRECTED: vanilla `AbstractHorse.registerGoals` does add `RandomLookAroundGoal` at 8 (line 141) and `RandomStandGoal` at 9 when rearing. The tverify-mobs3 claim that it is extra was wrong. No change. | AbstractHorse.java lines 136-152. |
| Wolf powder snow malus | entity/passive/wolf.rs | FIXED: vanilla constructor sets `POWDER_SNOW` -1. Added `set_pathfinding_malus(PathType::PowderSnow, -1.0)` in the constructor block. | Wolf.java lines 122-126. |
| Wolf `ON_TOP_OF_POWDER_SNOW` malus | entity/passive/wolf.rs | GAP: vanilla also sets `ON_TOP_OF_POWDER_SNOW` -1. `PathType` in `entity/ai/pathfinder/node.rs` has no such variant (only `PowderSnow` and `DangerPowderSnow`). Not added, needs a new PathType mapped to the pathfinder. | Wolf.java lines 124-125; node.rs lines 372-392. |
| Wolf goal list | entity/passive/wolf.rs | VERIFIED: 1 swim and panic (1.5, environmental), 2 sit, 3 llama avoid (24, 1.5, 1.5), 4 leap 0.4, 5 melee 1.0, 6 follow owner (1.0, 10, 2), 7 breed, 8 stroll 1.0, 9 beg 8, 10 look 8 and random look. Target: owner hurt-by 1, owner hurt 2, revenge with alert 3, player and anger reset 4 and 8, hunting 5-7. | Wolf.java lines 128-150. |
| Snow golem | entity/passive/snow_golem.rs | NOT RE-RUN: tverify-mobs3 already marks VERIFIED. Out of this pass's unverified list. | SnowGolem.java lines 60-82. |

Tests: none added. The wolf malus is a constructor side effect on the navigator, which needs a mob in a world to observe. A pure test would mirror the setter.

Checks: `rustfmt --edition 2024 crates/pumpkin-core/src/entity/passive/wolf.rs` exit 0. No cargo run (workers do not build).

Risks: the wolf malus edit is not compiled; it uses the same navigator lock pattern as `copper_golem.rs` and the `PathType` import path used there. The llama spit uncertainty and the wolf `ON_TOP_OF_POWDER_SNOW` path type remain open.
