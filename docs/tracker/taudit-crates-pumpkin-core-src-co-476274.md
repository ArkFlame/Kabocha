# command / ResourceKeyArgument (AUDIT)

Source reviewed: `crates/pumpkin-core/src/command/argument_types/resource_key.rs`
(branch `kab/taudit-crates-pumpkin-core-src-co-476274`).
Vanilla reference: `net.minecraft.commands.arguments.ResourceKeyArgument` (1.21.x). No `.vendor/`
decompile in this worktree, so parity was checked against the vanilla class behaviour and Pumpkin's
generated data.

Markers found: `TODO Recipe` (`resource_key.rs:37`), the "Vanilla has its own parser..." comment
(`resource_key.rs:76`). No `FIXME`.

## Fixed in this pass

- **Namespace ignored for advancements** (`resource_key.rs:113`). `Advancement::from_name` matches the
  path only, so `/advancement grant @s only custom:adventure/root` resolved the vanilla
  `minecraft:adventure/root`. Vanilla looks the full `ResourceLocation` up in the advancement
  registry and throws `advancement.advancementNotFound` for a foreign namespace. The lookup now goes
  through `Identifier::is_vanilla_then`.
- **Namespace ignored for biomes** (`resource_key.rs:128`, `get_biome`). Same class of bug:
  `custom:plains` resolved `minecraft:plains`. The namespace is checked before `Biome::from_name`.
- **Documentation** (`resource_key.rs:32-39`). Fixed the "juste" typo and removed the stale
  `TODO Recipe` marker; vanilla has no recipe `ResourceKeyArgument` factory (recipe commands use a
  plain resource location), so there is nothing to port.

## Remaining gaps

1. **Advancements resolve from the static generated list, not the server advancement manager**
   (`resource_key.rs:113`). Vanilla `ResourceKeyArgument.getAdvancement` does
   `server.getAdvancements().get(location)`, so datapack advancements are grantable. Pumpkin's
   `AdvancementManager::get_advancements` (`crates/pumpkin-core/src/data/advancement_data.rs:41`)
   also just returns `Advancement::get_identifier_list()`, so there is nowhere to resolve a datapack
   id from. Needs a runtime advancement registry. Size M.
2. **No suggestions for the dimension registry** (`resource_key.rs:60-72`). Vanilla builds a
   per-argument suggestion provider from the registry, so `/execute in`, `/clone from|to` and
   `/execute positioned ...` suggest dimension keys. Pumpkin returns `Suggestions::empty()` for
   everything except advancement and biome. Add a `DIMENSION_REGISTRY` branch over the server level
   keys (verify against vanilla source which set dimension() suggests before editing). Size S.
3. **Suggestion namespace stripping** (`resource_key.rs:62-69`). Pumpkin suggests full
   `minecraft:`-prefixed ids. Vanilla's `SharedSuggestionProvider.suggestResource` also offers the
   bare path when the remaining input has no `:` (unverified without a decompile). Client-visible
   only. Size S.
4. **Error text arity** (`resource_key.rs:19-30`, `103-132`). Vanilla throws
   `SimpleCommandExceptionType.create()` with no arguments for `advancement.advancementNotFound` and
   `commands.fillbiome.invalid`; Pumpkin declares `CommandErrorType<1>` and passes the identifier
   path, which the translation drops. Same wire text, but callers are forced to invent an argument.
   Consider `CommandErrorType<0>`. Size S.
5. **Dead duplicate constant** (`resource_key.rs:27-30`). `ERROR_NOT_SUMMONABLE_ENTITY` is unused
   here; the summon path keeps a private copy in
   `crates/pumpkin-command/src/argument_types/resource.rs:86`. Vanilla defines it in the same class
   because `getSummonableEntity` lives there. Delete it here or re-export it from the argument type
   that actually uses it. Size S.
