# tloot sweep: loot, item and recipe parity (hand-written Rust)

Status legend: FIXED (code changed, test added), UNVERIFIED (implemented from vanilla knowledge, no decompiled check run), OPEN (not changed).

| Area | Location | Vanilla (1.21.x) | Finding | Status |
|---|---|---|---|---|
| explosion_decay | world/loot/generation.rs `FurnaceSmelt`/`ExplosionDecay` arm | `ExplosionDecay` rolls `nextFloat() <= 1/radius` per item whenever the radius param is present | Rust skipped the roll when radius <= 1, so the RNG stream diverged | FIXED, UNVERIFIED |
| survives_explosion | world/loot.rs `check_condition` and `check_dynamic_condition` | `ExplosionCondition` rolls `nextFloat() <= 1/radius` unconditionally | Same short-circuit `radius <= 1.0 ||` removed; RNG consumed as vanilla | FIXED, UNVERIFIED; test `survives_explosion_always_consumes_one_roll` |
| uniform float provider | world/loot/generation.rs `number_float` | `Mth.randomBetween` = `nextFloat() * (max - min) + min`, always consumes one float | Rust returned `min` without rolling when min >= max | FIXED, UNVERIFIED |
| ore_drops (fortune) | world/loot.rs `apply_bonus_formula` | `level > 0 ? count * (max(0, nextInt(level+2)-1)+1) : count` | Formula matches; no RNG at level 0 | Matches; tests `ore_drops_*` |
| uniform_bonus_count | world/loot.rs `apply_bonus_formula` | `count + nextInt(multiplier * level + 1)` | Matches formula. Level 0 still draws `nextInt(1)` in Rust; vanilla level-0 guard not confirmed | OPEN, needs decompiled check |
| binomial_with_bonus_count | world/loot.rs `apply_bonus_formula` | loop of `level + extra` float rolls | Matches | Matches |
| set_count / limit_count | world/loot/generation.rs | `add` adds to stack count; clamp to min/max | Matches | Matches |
| enchanted_count (looting) | world/loot/generation.rs | `round(count * level)` added, optional limit | Rust computes `floor(f32 * level + 0.5)` in f64; same result for realistic values | Matches |
| loot pool rolls | world/loot/generation.rs `pool` | `rolls + floor(bonus_rolls * luck)` | Matches, rolls drawn before bonus | Matches |
| weighted entry pick | world/loot/generation.rs `pool` | `nextInt(total)` then subtract weights | Rust skips the draw when only one candidate exists; vanilla single-entry path not confirmed | OPEN, needs decompiled check |
| mending xp to durability | entity/player.rs (repair from xp) | `durabilityToXp(d) = d / 2` (floor), `xpToDurability(x) = 2x` | Rust used `(repaired + 1) / 2` (ceil) and over-charged xp | FIXED, UNVERIFIED, no unit test (needs player harness) |
| unbreaking / armor durability | pumpkin-data item_stack `should_apply_durability_damage_with`, generated `modify_durability_damage` | armor applies with 0.6 + 0.4/(lvl+1); others 1/(lvl+1) | Probabilities live in datapack effect data (`remove_binomial`), which is out of scope (assets). Not changed | OPEN, data check needed |
| enchant cost tables | not found in hand-written item/ or enchantment/ code | enchanting table cost tables | No hand-written cost formula located in this sweep | OPEN |
