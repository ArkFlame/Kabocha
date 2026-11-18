# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-block-bloc.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

SnowyBlock.java, GrassBlock.java, MyceliumBlock.java) and every STILL OPEN row. All four rows are now closed
so no STILL OPEN row remains.
were needed; the code already matches vanilla. No STILL OPEN row remains.
and no STILL OPEN row remains.

## Result
All four rows verified against decompiled vanilla 1.21 in
``.vendor/vanilla-decompiled/net/minecraft/world/level/block/``. No code changes were
needed: Pumpkin already matches vanilla, so nothing to rustfmt. No row is BLOCKED-DATA.

- Row 1 (on_place / neighbor update on grass, mycelium, podzol): VERIFIED. Vanilla
  ``SnowyBlock.getStateForPlacement`` (SnowyBlock.java:41-44) and ``updateShape`` (:24-38)
  set SNOWY from the block above via ``isSnowySetting`` (BlockTags.SNOW, :46-48).
  GrassBlock and MyceliumBlock extend SpreadingSnowyBlock extends SnowyBlock and
  override neither method (grep of both files confirms); Podzol is a plain SnowyBlock.
  Pumpkin's ``SnowyBlock::on_place`` (spreading_snowy_block.rs:29-34) and
  ``get_state_for_neighbor_update`` (:37-49) mirror this, so on_place stays on all three.
- Row 2 (fluid amount generic; :58-63): VERIFIED. Vanilla ``FluidState.isFull()`` is
  ``getAmount() == 8``, used at SpreadingSnowyBlock.java:37. Pumpkin ``is_full_fluid``
  checks ``!is_empty && level == 8``; ``fluid_state_from_block_state``
  (crates/pumpkin-world/src/world.rs:140-182) reads the LEVEL property of any liquid
  block generically and maps source/falling to amount 8. No WATER/LAVA special case.
- Row 3 (LAYERS == 1 short-circuit and light block; :71-94): VERIFIED. Vanilla
  ``canStayAlive`` (SpreadingSnowyBlock.java:29-43) short-circuits on SNOW && LAYERS == 1
  (:33-35), rejects a full fluid (:37-39), then returns ``getLightDampeningInto(state,
  aboveState, UP, aboveState.getLightDampening()) < 15`` (:41-42). Pumpkin
  ``can_stay_alive_with_above`` matches every branch, mapping light dampening to
  ``BlockState::opacity``; four pure unit tests cover it (:248-289).
- Row 4 (randomTick coverage; :116-179): VERIFIED against SpreadingSnowyBlock.java:56-104.
  The fade-on-!canStayAlive branch (:68-75), ``getMaxLocalRawBrightness(above) >= 9`` gate
  (:77), 4-offset spread loop (:80-81), ``defaultBlockState`` passed to canPropagate (:95),
  and SNOWY payload (:96) all match. The ``pos == testPos`` Canvas skip (:83) is redundant
  because a spreading block is never its own base block. Live-world fade/spread/survival
  is covered by ``test_spreading_snowy_grass_random_tick``
  (crates/pumpkin-core/src/server/builtin_game_tests/plants_and_support_blocks.rs:316-361).

No STILL OPEN row remains and no BLOCKED-DATA row exists: every value comes from
existing generated data. No ``.rs`` files were edited this pass.
