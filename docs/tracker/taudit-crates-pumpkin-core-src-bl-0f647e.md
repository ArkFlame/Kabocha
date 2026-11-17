# Audit: crates/pumpkin-core/src/block/entities/piston.rs

Rows: TODO/FIXME markers: none in file. The 5 "vanilla" markers are approximation comments at lines 55, 68, 89, 103, 163.
Vanilla class: net.minecraft.world.level.block.piston.PistonMovingBlockEntity (+ PistonHeadBlock shapes).

## Fixed in this pass
- facing NBT type: vanilla \`saveAdditional\` writes \`facing\` with \`putInt\`, \`load\` reads \`getInt\`; Pumpkin wrote/read a Byte tag and NbtCompound::get_byte rejects Int tags, so every vanilla-written chunk reset the facing to Down. Now reads numeric with byte fallback and writes an Int. Test: facing_is_loaded_from_vanilla_int_tag.
- finish() diverged from vanilla \`finalTick\`: it placed the \`updateFromNeighbourShapes\` result directly, so a moved block that cannot survive at its destination vanished without drops and a waterlogged moved block kept WATERLOGGED. tick() already had the vanilla updateOrDestroy + waterlogged-clear path; both now share \`place_final_state\`.

## Remaining gaps
1. piston.rs:57-112 \`push_entities\` sweeps a full block cube (or a hardcoded 4px slab for source=true) instead of \`movedState.getCollisionShape(level, position)\` used by vanilla \`PistonMovingBlockEntity#getCollidedEntities\`. Fences, walls, doors, slabs, chests and short/long piston heads therefore push entities in the wrong volume. Fix: compute the swept box from the moved block's real collision shape (needs a shape->AABB accessor on BlockState). Size M.
2. piston.rs:100-101 push distance uses \`intersection.min(delta) + 0.01\`; vanilla additionally moves the entity along \`movementDirection\` and, for players, sends the delta so client prediction does not fight the server. Only non-player entities are moved here. Size S/M.
3. piston.rs:89-93 players are skipped entirely, but vanilla pushes players with \`Entity#move\` (pistons push players in vanilla). Deliberate Kabocha choice (client-authoritative movement); revisit only if player pushing is wanted. Size M (needs movement packet).
4. piston.rs:139-161 \`push_out_of_piston_body\` has no vanilla counterpart; vanilla relies on \`Entity#move\` collision resolution after the piston move. Extra shove can double-displace entities in tight spots. Size S.
5. piston.rs:163-178 head slab thickness is a constant 0.25 for every facing, ignoring the head block state (\`SHORT\` arm length) and \`PistonHeadBlock#getShape\`'s 4px box computed from the facing. Correct for the default head, wrong for short heads. Size S.
6. piston.rs:310-312 \`update_packet_nbt\` returns None, so clients never receive \`blockState\`/facing/progress/extending for the moving piston BE. Vanilla clients render the moving block from that BE data (PistonBlockEntityRenderer); Pumpkin relies on the client animating from the block event alone, which loses the moved block's appearance. Client-only effect; Size M.
7. piston.rs:278-282 \`from_nbt\` accepts any progress (NaN/>1 leak into animation) and falls back to \`BlockDirection::Down\` on an out-of-range facing where vanilla \`Direction.from3DDataValue\` clamps to Down too (that part matches); clamping progress to 0.0..=1.0 would be defensive. Size S.
