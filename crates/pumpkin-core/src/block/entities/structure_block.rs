use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use pumpkin_data::block_properties::{StructureBlockLikeProperties, StructureblockMode};
use pumpkin_data::{Block, BlockDirection, BlockStateId, Mirror, Rotation};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_nbt::nbt_compress::write_gzip_compound_tag_to_bytes;
use pumpkin_nbt::tag::NbtTag;
use pumpkin_util::math::block_box::BlockBox;
use pumpkin_util::math::boundingbox::BoundingBox;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector2::Vector2;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::math::wrap_degrees;
use pumpkin_util::random::{RandomImpl, legacy_rand::LegacyRand};
use pumpkin_world::chunk::palette::BlockPalette;
use pumpkin_world::generation::structure::template::{
    BlockPlacer, BlockStateResolver, Palette, PaletteEntry, StructureBlockInfo,
    StructureEntityInfo, StructureTemplate, global_cache,
};
use pumpkin_world::world::BlockFlags;

use crate::entity::r#type::{load_entity_from_nbt, self_and_passengers};
use crate::world::World;
use crate::world::block_placer::WorldBlockPlacer;

use super::BlockEntity;

const SCAN_CORNER_BLOCKS_RANGE: i32 = 80;
pub const MAX_OFFSET_PER_AXIS: i32 = 48;
pub const MAX_SIZE_PER_AXIS: i32 = 48;
const DEFAULT_POS_X: i32 = 0;
const DEFAULT_POS_Y: i32 = 1;
const DEFAULT_POS_Z: i32 = 0;

#[must_use]
pub const fn mode_to_legacy_name(mode: StructureblockMode) -> &'static str {
    match mode {
        StructureblockMode::Save => "SAVE",
        StructureblockMode::Load => "LOAD",
        StructureblockMode::Corner => "CORNER",
        StructureblockMode::Data => "DATA",
    }
}

#[must_use]
pub const fn mode_from_name(name: &str) -> Option<StructureblockMode> {
    if name.eq_ignore_ascii_case("SAVE") {
        Some(StructureblockMode::Save)
    } else if name.eq_ignore_ascii_case("LOAD") {
        Some(StructureblockMode::Load)
    } else if name.eq_ignore_ascii_case("CORNER") {
        Some(StructureblockMode::Corner)
    } else if name.eq_ignore_ascii_case("DATA") {
        Some(StructureblockMode::Data)
    } else {
        None
    }
}

#[must_use]
pub const fn mode_from_packet_id(id: i32) -> StructureblockMode {
    match id {
        0 => StructureblockMode::Save,
        1 => StructureblockMode::Load,
        2 => StructureblockMode::Corner,
        _ => StructureblockMode::Data,
    }
}

#[must_use]
pub const fn mirror_to_legacy_name(mirror: Mirror) -> &'static str {
    match mirror {
        Mirror::None => "NONE",
        Mirror::LeftRight => "LEFT_RIGHT",
        Mirror::FrontBack => "FRONT_BACK",
    }
}

#[must_use]
pub const fn mirror_from_name(name: &str) -> Mirror {
    if name.eq_ignore_ascii_case("LEFT_RIGHT") {
        Mirror::LeftRight
    } else if name.eq_ignore_ascii_case("FRONT_BACK") {
        Mirror::FrontBack
    } else {
        Mirror::None
    }
}

#[must_use]
pub const fn mirror_from_packet_id(id: i32) -> Mirror {
    match id {
        1 => Mirror::LeftRight,
        2 => Mirror::FrontBack,
        _ => Mirror::None,
    }
}

#[must_use]
pub const fn rotation_to_legacy_name(rotation: Rotation) -> &'static str {
    match rotation {
        Rotation::None => "NONE",
        Rotation::Clockwise90 => "CLOCKWISE_90",
        Rotation::Rotate180 => "CLOCKWISE_180",
        Rotation::CounterClockwise90 => "COUNTERCLOCKWISE_90",
    }
}

#[must_use]
pub fn rotation_from_name(name: &str) -> Rotation {
    if name.eq_ignore_ascii_case("CLOCKWISE_90") {
        Rotation::Clockwise90
    } else if name.eq_ignore_ascii_case("CLOCKWISE_180") || name == "180" {
        Rotation::Rotate180
    } else if name.eq_ignore_ascii_case("COUNTERCLOCKWISE_90") {
        Rotation::CounterClockwise90
    } else {
        Rotation::None
    }
}

#[must_use]
pub const fn rotation_from_packet_id(id: i32) -> Rotation {
    match id.rem_euclid(4) {
        1 => Rotation::Clockwise90,
        2 => Rotation::Rotate180,
        3 => Rotation::CounterClockwise90,
        _ => Rotation::None,
    }
}

/// Parses and validates a vanilla `Identifier` string into `(namespace, path, canonical_id)`.
#[must_use]
pub fn parse_structure_identifier(raw: &str) -> Option<(String, String, String)> {
    if raw.is_empty() {
        return None;
    }
    let (namespace, path) = raw.split_once(':').unwrap_or(("minecraft", raw));
    if namespace.is_empty() || path.is_empty() {
        return None;
    }
    if !namespace
        .bytes()
        .all(|b| matches!(b, b'a'..=b'z' | b'0'..=b'9' | b'_' | b'.' | b'-'))
    {
        return None;
    }
    if !path
        .bytes()
        .all(|b| matches!(b, b'a'..=b'z' | b'0'..=b'9' | b'_' | b'.' | b'-' | b'/'))
    {
        return None;
    }
    let canonical = format!("{namespace}:{path}");
    Some((namespace.to_string(), path.to_string(), canonical))
}

/// Validates that a structure `path` cannot traverse directories when mapped under `generated/`.
#[must_use]
pub fn is_safe_generated_structure_path(path: &str) -> bool {
    !path.is_empty()
        && path
            .split('/')
            .all(|seg| !seg.is_empty() && seg != "." && seg != "..")
}

#[must_use]
pub fn generated_structure_file_path(
    world: &World,
    namespace: &str,
    path: &str,
) -> Option<PathBuf> {
    if !is_safe_generated_structure_path(path) {
        return None;
    }
    let mut full_path = world
        .level
        .level_folder
        .root_folder
        .join("generated")
        .join(namespace)
        .join("structure");
    for segment in path.split('/') {
        full_path.push(segment);
    }
    full_path.as_mut_os_string().push(".nbt");
    Some(full_path)
}

// Vanilla marks these blocks dynamicShape() in Blocks (1.21.x): their collision
// shape is state-dependent, so StructureTemplate.addToLists buckets them as other
// even when one state's collision box is a full cube.
#[must_use]
fn has_dynamic_shape(name: &str) -> bool {
    matches!(
        name,
        "moving_piston"
            | "bamboo"
            | "scaffolding"
            | "powder_snow"
            | "pointed_dripstone"
            | "sulfur_spike"
            | "shulker_box"
            | "white_shulker_box"
            | "orange_shulker_box"
            | "magenta_shulker_box"
            | "light_blue_shulker_box"
            | "yellow_shulker_box"
            | "lime_shulker_box"
            | "pink_shulker_box"
            | "gray_shulker_box"
            | "light_gray_shulker_box"
            | "cyan_shulker_box"
            | "purple_shulker_box"
            | "blue_shulker_box"
            | "brown_shulker_box"
            | "green_shulker_box"
            | "red_shulker_box"
            | "black_shulker_box"
    )
}

pub fn capture_structure_template_from_world(
    world: &World,
    origin: BlockPos,
    size: Vector3<i32>,
    author: &str,
    ignored_blocks: &[&'static Block],
    include_entities: bool,
) -> StructureTemplate {
    let mut template = StructureTemplate::default();
    template.set_author(author.to_string());
    if size.x < 1 || size.y < 1 || size.z < 1 {
        return template;
    }

    template.size = size;
    let end = BlockPos::new(
        origin.0.x + size.x - 1,
        origin.0.y + size.y - 1,
        origin.0.z + size.z - 1,
    );
    let min_x = origin.0.x.min(end.0.x);
    let min_y = origin.0.y.min(end.0.y);
    let min_z = origin.0.z.min(end.0.z);
    let max_x = origin.0.x.max(end.0.x);
    let max_y = origin.0.y.max(end.0.y);
    let max_z = origin.0.z.max(end.0.z);

    let mut full_blocks = Vec::new();
    let mut other_blocks = Vec::new();
    let mut block_entity_blocks = Vec::new();

    for y in min_y..=max_y {
        for x in min_x..=max_x {
            for z in min_z..=max_z {
                let world_pos = BlockPos::new(x, y, z);
                let (block, state) = world.get_block_and_state(&world_pos);
                if block == &Block::STRUCTURE_VOID || ignored_blocks.contains(&block) {
                    continue;
                }

                let block_name = format!("minecraft:{}", block.name);
                let props = block
                    .properties(state.id)
                    .map(|p| p.to_props())
                    .unwrap_or_default();
                let mut entry = PaletteEntry::with_properties(block_name, props);
                entry.state_id = Some(state.id);

                let nbt = world.get_block_entity(&world_pos).map(|be| {
                    let mut tag = NbtCompound::new();
                    tag.put_string("id", be.resource_location().to_string());
                    be.write_nbt(&mut tag);
                    Box::new(tag)
                });

                let rel_pos = Vector3::new(x - min_x, y - min_y, z - min_z);
                let info = StructureBlockInfo::new(rel_pos, Arc::new(entry), nbt);
                if info.nbt.is_some() {
                    block_entity_blocks.push(info);
                } else if state.is_full_cube() && !has_dynamic_shape(block.name) {
                    // Vanilla: !hasDynamicShape() && isCollisionShapeFullBlock(...),
                    // not render occlusion (glass is full-collision).
                    full_blocks.push(info);
                } else {
                    other_blocks.push(info);
                }
            }
        }
    }

    let mut ordered =
        Vec::with_capacity(full_blocks.len() + other_blocks.len() + block_entity_blocks.len());
    ordered.extend(full_blocks);
    ordered.extend(other_blocks);
    ordered.extend(block_entity_blocks);
    template.palettes = vec![Palette::new(ordered)];

    // Vanilla StructureTemplate#fillFromWorld calls fillEntityList for live entities
    // touching the captured box, excluding players, unless Ignore Entities is set.
    if include_entities {
        // Vanilla AABB.encapsulatingFullBlocks: the box spans each block cell fully.
        let area = BoundingBox::new(
            Vector3::new(min_x as f64, min_y as f64, min_z as f64),
            Vector3::new(max_x as f64 + 1.0, max_y as f64 + 1.0, max_z as f64 + 1.0),
        );
        for entity in world.get_entities_at_box(&area) {
            let entity_pos = entity.get_entity().pos.load();
            let rel_pos = Vector3::new(
                entity_pos.x - min_x as f64,
                entity_pos.y - min_y as f64,
                entity_pos.z - min_z as f64,
            );

            let mut nbt = NbtCompound::new();
            entity.write_nbt(&mut nbt);

            // Vanilla: paintings use their attached block pos; other entities use
            // BlockPos.containing(pos). Pumpkin stores that attached pos as block_pos.
            let block_pos = nbt
                .get_int_array("block_pos")
                .and_then(|a| <[i32; 3]>::try_from(a).ok())
                .map_or_else(
                    || BlockPos::floored_v(rel_pos).0,
                    |[x, y, z]| Vector3::new(x - min_x, y - min_y, z - min_z),
                );

            template
                .entity_info_list
                .push(StructureEntityInfo::new(rel_pos, block_pos, nbt));
        }
    }

    template
}

pub fn save_structure_from_world(
    world: &World,
    raw_identifier: &str,
    origin: BlockPos,
    size: Vector3<i32>,
    author: &str,
    save_to_disk: bool,
    ignored_blocks: &[&'static Block],
    include_entities: bool,
) -> bool {
    let Some((namespace, path, canonical_id)) = parse_structure_identifier(raw_identifier) else {
        return false;
    };
    if !is_safe_generated_structure_path(&path) {
        return false;
    }

    let template = capture_structure_template_from_world(
        world,
        origin,
        size,
        author,
        ignored_blocks,
        include_entities,
    );
    let nbt = template.save();
    let Ok(bytes) = write_gzip_compound_tag_to_bytes(nbt) else {
        return false;
    };

    global_cache().register_template(&canonical_id, Arc::from(bytes.as_slice()));

    if save_to_disk {
        let Some(file_path) = generated_structure_file_path(world, &namespace, &path) else {
            return false;
        };
        if let Some(parent) = file_path.parent()
            && std::fs::create_dir_all(parent).is_err()
        {
            return false;
        }
        if std::fs::write(file_path, &bytes).is_err() {
            return false;
        }
    }

    true
}

#[must_use]
pub fn load_or_cache_structure_template(
    world: &World,
    raw_identifier: &str,
) -> Option<Arc<StructureTemplate>> {
    let (namespace, path, canonical_id) = parse_structure_identifier(raw_identifier)?;
    if !is_safe_generated_structure_path(&path) {
        return None;
    }

    if !global_cache().has_dynamic(&canonical_id)
        && let Some(file_path) = generated_structure_file_path(world, &namespace, &path)
        && file_path.is_file()
        && let Ok(bytes) = std::fs::read(&file_path)
    {
        global_cache().register_template(&canonical_id, Arc::from(bytes.as_slice()));
    }

    global_cache().get(&canonical_id)
}

/// Block entity ids whose vanilla class implements `RandomizableContainer`, the only ones that
/// receive a `LootTableSeed` when a template is placed.
fn is_randomizable_container_id(id: &str) -> bool {
    let path = id.strip_prefix("minecraft:").unwrap_or(id);
    matches!(
        path,
        "chest"
            | "trapped_chest"
            | "barrel"
            | "dispenser"
            | "dropper"
            | "hopper"
            | "crafter"
            | "decorated_pot"
            | "suspicious_sand"
            | "suspicious_gravel"
    ) || path.ends_with("shulker_box")
}

fn create_structure_seed(seed: i64) -> u64 {
    if seed == 0 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |d| d.as_millis() as u64)
    } else {
        seed as u64
    }
}

/// Rewrites a template entity's `block_pos` to its transformed world position.
/// Vanilla stores this through `BlockPos.CODEC` (three ints); Pumpkin's IntArray matches that shape.
fn put_transformed_block_pos(nbt: &mut NbtCompound, pos: Vector3<i32>) {
    if nbt.child_tags.contains_key("block_pos") {
        nbt.put("block_pos", NbtTag::IntArray(vec![pos.x, pos.y, pos.z]));
    }
}

/// Spawns every entity stored in a template after its blocks are placed.
///
/// Vanilla StructureTemplate#placeEntities transforms each entry's block and
/// exact positions, rewrites Pos and block_pos, drops the old UUID, applies
/// the mirror and rotation to the stored yaw and creates the entity from NBT.
fn place_template_entities(
    world: &Arc<World>,
    template: &StructureTemplate,
    origin: BlockPos,
    pivot: Vector3<i32>,
    mirror: Mirror,
    rotation: Rotation,
) {
    for entity_info in &template.entity_info_list {
        let block_pos =
            StructureTemplate::transform_block_pos(entity_info.block_pos, mirror, rotation, pivot);
        let rel = StructureTemplate::transform_vec3(entity_info.pos, mirror, rotation, pivot);
        let pos = Vector3::new(
            origin.0.x as f64 + rel.x,
            origin.0.y as f64 + rel.y,
            origin.0.z as f64 + rel.z,
        );

        let mut nbt = entity_info.nbt.clone();
        nbt.put_list(
            "Pos",
            vec![
                NbtTag::Double(pos.x),
                NbtTag::Double(pos.y),
                NbtTag::Double(pos.z),
            ],
        );
        nbt.child_tags.remove("UUID");
        put_transformed_block_pos(
            &mut nbt,
            Vector3::new(
                origin.0.x + block_pos.x,
                origin.0.y + block_pos.y,
                origin.0.z + block_pos.z,
            ),
        );
        if let Some(rotation_list) = nbt.get_list("Rotation")
            && rotation_list.len() >= 2
            && let Some(yaw) = rotation_list[0].extract_float()
            && let Some(pitch) = rotation_list[1].extract_float()
        {
            let yaw = transform_entity_yaw(yaw, mirror, rotation);
            nbt.put_list("Rotation", vec![NbtTag::Float(yaw), NbtTag::Float(pitch)]);
        }

        if let Some(root) = load_entity_from_nbt(&nbt, pos, world) {
            for entity in self_and_passengers(&root) {
                world.spawn_entity(entity);
            }
        }
    }
}

/// Vanilla Entity#rotate plus Entity#mirror as applied by
/// StructureTemplate#placeEntities to a template entity's stored yaw.
fn transform_entity_yaw(yaw: f32, mirror: Mirror, rotation: Rotation) -> f32 {
    let wrapped = wrap_degrees(yaw);
    let rotated = wrapped
        + match rotation {
            Rotation::Clockwise90 => 90.0,
            Rotation::CounterClockwise90 => 270.0,
            Rotation::Rotate180 => 180.0,
            Rotation::None => 0.0,
        };
    let mirrored = match mirror {
        Mirror::LeftRight => 180.0 - wrapped,
        Mirror::FrontBack => -wrapped,
        Mirror::None => wrapped,
    };
    rotated + mirrored - yaw
}

#[derive(Clone, Copy, Debug)]
pub struct StructurePlacementSettings {
    pub mirror: Mirror,
    pub rotation: Rotation,
    pub integrity: f32,
    pub seed: i64,
    pub strict: bool,
    pub ignore_entities: bool,
}

impl Default for StructurePlacementSettings {
    fn default() -> Self {
        Self {
            mirror: Mirror::None,
            rotation: Rotation::None,
            integrity: 1.0,
            seed: 0,
            strict: false,
            ignore_entities: false,
        }
    }
}

/// Returns the cell just outside the placed shape in `direction`, or `None` when the neighbour
/// is also part of the shape (an interior face that vanilla `forAllFaces` does not visit).
fn edge_neighbor(
    placed: &HashSet<BlockPos>,
    pos: &BlockPos,
    direction: BlockDirection,
) -> Option<BlockPos> {
    let neighbor = pos.offset(direction.to_offset());
    (!placed.contains(&neighbor)).then_some(neighbor)
}

/// Mirrors vanilla `StructureTemplate.updateShapeAtEdge`: for every face on the boundary of the
/// placed voxel shape, reshape both the placed block and the block just outside the structure so
/// fences, walls, stairs and attached blocks reconnect to their new neighbours.
fn update_shape_at_edge(world: &Arc<World>, placed: &[(BlockPos, BlockStateId)]) {
    let placed_positions: HashSet<BlockPos> = placed.iter().map(|(pos, _)| *pos).collect();
    for (pos, _) in placed {
        for direction in BlockDirection::all() {
            let Some(neighbor_pos) = edge_neighbor(&placed_positions, pos, direction) else {
                continue;
            };

            let state_id = world.get_block_state_id(pos);
            let neighbor_state_id = world.get_block_state_id(&neighbor_pos);

            let state = Block::from_state_id(state_id);
            let new_state_id = world.block_registry.get_state_for_neighbor_update(
                world,
                &state,
                state_id,
                pos,
                direction,
                &neighbor_pos,
                neighbor_state_id,
            );
            if new_state_id != state_id {
                world.set_block_state(pos, new_state_id, BlockFlags::NOTIFY_LISTENERS);
            }

            let neighbor_state = Block::from_state_id(neighbor_state_id);
            let new_neighbor_state_id = world.block_registry.get_state_for_neighbor_update(
                world,
                &neighbor_state,
                neighbor_state_id,
                &neighbor_pos,
                direction.opposite(),
                pos,
                new_state_id,
            );
            if new_neighbor_state_id != neighbor_state_id {
                world.set_block_state(
                    &neighbor_pos,
                    new_neighbor_state_id,
                    BlockFlags::NOTIFY_LISTENERS,
                );
            }
        }
    }
}

pub fn place_structure_template_in_world(
    world: &Arc<World>,
    template: &StructureTemplate,
    origin: BlockPos,
    settings: &StructurePlacementSettings,
) -> bool {
    let StructurePlacementSettings {
        mirror,
        rotation,
        integrity,
        seed,
        strict,
        ignore_entities,
    } = *settings;
    if template.size.x < 1 || template.size.y < 1 || template.size.z < 1 {
        return false;
    }
    let Some(palette) = template.palettes().first() else {
        return false;
    };
    if palette.blocks().is_empty() && (ignore_entities || template.entity_info_list.is_empty()) {
        return false;
    }

    let clamped_integrity = if integrity.is_nan() {
        0.0
    } else {
        integrity.clamp(0.0, 1.0)
    };
    // Vanilla builds two independent streams from the same seed: the structure
    // settings random drives the integrity processor and the placement random
    // drives loot table seeds, so neither affects the other's sequence.
    let mut integrity_rng = LegacyRand::from_seed(create_structure_seed(seed));
    let mut loot_rng = LegacyRand::from_seed(create_structure_seed(seed));
    // Strict placement maps to vanilla UPDATE_SKIP_ALL_SIDEEFFECTS: the known-shape half is
    // handled by the !strict reconnect pass below, the drop-callback half by the placer flag
    // (the block entity pre-clear above matches the flag vanilla also adds in non-strict mode).
    let mut placer = WorldBlockPlacer::new_with_skip_side_effects(world, strict);
    let pivot = Vector3::new(0, 0, 0);

    for block_info in palette.blocks() {
        let Some(resolved_state) = BlockStateResolver::resolve(&block_info.state, rotation, mirror)
        else {
            continue;
        };

        let rel = StructureTemplate::transform_block_pos(block_info.pos, mirror, rotation, pivot);
        let world_vec = Vector3::new(origin.0.x + rel.x, origin.0.y + rel.y, origin.0.z + rel.z);
        let world_pos = BlockPos(world_vec);
        if !world.is_in_build_limit(world_pos) {
            continue;
        }

        // Vanilla BlockRotProcessor applies to every info (with or without NBT) and
        // drops the rejected entry, leaving any block already at that position intact.
        if clamped_integrity < 1.0 && integrity_rng.next_f32() > clamped_integrity {
            continue;
        }
        let state = resolved_state;

        // Vanilla pre-sets a BARRIER with UPDATE_SKIP_ALL_SIDEEFFECTS before placing an entry
        // that carries block entity NBT: that clears the block entity at the position silently.
        // Entries without NBT use a plain replacement, so a non-strict placement still drops
        // the replaced container's contents.
        if block_info.nbt.is_some() {
            world.remove_block_entity(&world_pos);
        }
        placer.set_block_state(&world_vec, state);

        if let Some(template_nbt) = &block_info.nbt {
            let final_block = Block::from_id(resolved_state.id.to_block_id());
            let mut placed_nbt = NbtCompound::new();
            let id = template_nbt
                .get_string("id")
                .map_or_else(|| format!("minecraft:{}", final_block.name), str::to_string);
            placed_nbt.put_string("id", id.clone());
            placed_nbt.put_int("x", world_vec.x);
            placed_nbt.put_int("y", world_vec.y);
            placed_nbt.put_int("z", world_vec.z);
            for (key, value) in &template_nbt.child_tags {
                if key.as_ref() != "x" && key.as_ref() != "y" && key.as_ref() != "z" {
                    placed_nbt.child_tags.insert(key.clone(), value.clone());
                }
            }
            // Vanilla `StructureTemplate.placeInWorld` seeds every RandomizableContainer it
            // places, advancing the placement RNG even when the block has no LootTable tag.
            if is_randomizable_container_id(&id) {
                placed_nbt.put_long("LootTableSeed", loot_rng.next_i64());
            }
            placer.add_block_entity(placed_nbt);
        }
    }

    placer.finalize();
    for nbt in &placer.block_entity_nbts {
        if let (Some(x), Some(y), Some(z)) = (nbt.get_int("x"), nbt.get_int("y"), nbt.get_int("z"))
            && let Some(be) = world.get_block_entity(&BlockPos::new(x, y, z))
        {
            world.update_block_entity(&be);
        }
    }
    world.queue_block_updates(&placer.changed_positions);

    if !strict {
        // Vanilla reconnects multi-block states (fences, stairs, walls, ...) once the whole
        // template is written, via Block.updateFromNeighbourShapes, then notifies neighbours.
        // Strict placement sets UPDATE_KNOWN_SHAPE and skips both passes.
        // updateShapeAtEdge runs first, covering blocks just outside the placed box.
        update_shape_at_edge(world, &placer.changed_positions);
        for (changed_pos, _placed_state_id) in &placer.changed_positions {
            let current_state_id = world.get_block_state_id(changed_pos);
            let shaped_state_id = world.update_from_neighbor_shapes(current_state_id, changed_pos);
            if shaped_state_id != current_state_id {
                world.set_block_state(
                    changed_pos,
                    shaped_state_id,
                    BlockFlags::NOTIFY_LISTENERS | BlockFlags::UPDATE_KNOWN_SHAPE,
                );
            }
            let block = Block::from_state_id(shaped_state_id);
            world.update_neighbors_at(changed_pos, block, None);
        }
    }

    world.flush_block_updates();

    if !ignore_entities {
        place_template_entities(world, template, origin, pivot, mirror, rotation);
    }

    true
}

/// Computes `(offset, size)` from matching corner positions using vanilla `StructureBlockEntity.detectSize` rules.
#[must_use]
pub fn compute_detected_bounds(
    block_pos: BlockPos,
    corners: &[BlockPos],
) -> Option<(Vector3<i32>, Vector3<i32>)> {
    if corners.is_empty() {
        return None;
    }

    let first = corners[0];
    let mut bbox = BlockBox::new(
        first.0.x, first.0.y, first.0.z, first.0.x, first.0.y, first.0.z,
    );
    if corners.len() == 1 {
        bbox.encompass(&BlockBox::new(
            block_pos.0.x,
            block_pos.0.y,
            block_pos.0.z,
            block_pos.0.x,
            block_pos.0.y,
            block_pos.0.z,
        ));
    } else {
        for corner in &corners[1..] {
            bbox.encompass(&BlockBox::new(
                corner.0.x, corner.0.y, corner.0.z, corner.0.x, corner.0.y, corner.0.z,
            ));
        }
    }

    let dx = bbox.max.x - bbox.min.x;
    let dy = bbox.max.y - bbox.min.y;
    let dz = bbox.max.z - bbox.min.z;
    if dx <= 1 || dy <= 1 || dz <= 1 {
        return None;
    }

    let offset = Vector3::new(
        bbox.min.x - block_pos.0.x + 1,
        bbox.min.y - block_pos.0.y + 1,
        bbox.min.z - block_pos.0.z + 1,
    );
    let size = Vector3::new(dx - 1, dy - 1, dz - 1);
    Some((offset, size))
}

fn is_structure_block_state(state: BlockStateId) -> bool {
    Block::from_state_id(state) == &Block::STRUCTURE_BLOCK
}

/// Vanilla `StructureBlockEntity#getEnclosingBoundingBox` walks block states, not block
/// entities, so a position only counts when its state is a structure block.
fn collect_structure_block_positions(
    sections: &[BlockPalette],
    chunk_x: i32,
    chunk_z: i32,
    base_y: i32,
    min_y: i32,
    max_y: i32,
) -> Vec<BlockPos> {
    let mut found = Vec::new();
    for (index, palette) in sections.iter().enumerate() {
        if !palette.contains_matching(is_structure_block_state) {
            continue;
        }
        let section_min_y = base_y + index as i32 * BlockPalette::SIZE as i32;
        for y in 0..BlockPalette::SIZE {
            let world_y = section_min_y + y as i32;
            if world_y < min_y || world_y > max_y {
                continue;
            }
            for z in 0..BlockPalette::SIZE {
                for x in 0..BlockPalette::SIZE {
                    if !is_structure_block_state(palette.get(x, y, z)) {
                        continue;
                    }
                    found.push(BlockPos(Vector3::new(
                        chunk_x * BlockPalette::SIZE as i32 + x as i32,
                        world_y,
                        chunk_z * BlockPalette::SIZE as i32 + z as i32,
                    )));
                }
            }
        }
    }
    found
}

pub struct StructureBlockBlockEntity {
    pub position: BlockPos,
    pub name: Mutex<String>,
    pub author: Mutex<String>,
    pub metadata: Mutex<String>,
    pub pos_x: Mutex<i32>,
    pub pos_y: Mutex<i32>,
    pub pos_z: Mutex<i32>,
    pub size_x: Mutex<i32>,
    pub size_y: Mutex<i32>,
    pub size_z: Mutex<i32>,
    pub rotation: Mutex<String>,
    pub mirror: Mutex<String>,
    pub mode: Mutex<String>,
    pub ignore_entities: Mutex<bool>,
    pub strict: Mutex<bool>,
    pub powered: Mutex<bool>,
    pub show_air: Mutex<bool>,
    pub show_bounding_box: Mutex<bool>,
    pub integrity: Mutex<f32>,
    pub seed: Mutex<i64>,
    pub dirty: AtomicBool,
}

impl BlockEntity for StructureBlockBlockEntity {
    fn resource_location(&self) -> &'static str {
        Self::ID
    }

    fn get_position(&self) -> BlockPos {
        self.position
    }

    fn from_nbt(nbt: &NbtCompound, position: BlockPos) -> Self
    where
        Self: Sized,
    {
        let raw_name = nbt.get_string("name").unwrap_or("");
        let canonical_name = parse_structure_identifier(raw_name)
            .map(|(_, _, id)| id)
            .unwrap_or_default();
        let pos_x = nbt
            .get_int("posX")
            .unwrap_or(DEFAULT_POS_X)
            .clamp(-MAX_OFFSET_PER_AXIS, MAX_OFFSET_PER_AXIS);
        let pos_y = nbt
            .get_int("posY")
            .unwrap_or(DEFAULT_POS_Y)
            .clamp(-MAX_OFFSET_PER_AXIS, MAX_OFFSET_PER_AXIS);
        let pos_z = nbt
            .get_int("posZ")
            .unwrap_or(DEFAULT_POS_Z)
            .clamp(-MAX_OFFSET_PER_AXIS, MAX_OFFSET_PER_AXIS);
        let size_x = nbt
            .get_int("sizeX")
            .unwrap_or(0)
            .clamp(0, MAX_SIZE_PER_AXIS);
        let size_y = nbt
            .get_int("sizeY")
            .unwrap_or(0)
            .clamp(0, MAX_SIZE_PER_AXIS);
        let size_z = nbt
            .get_int("sizeZ")
            .unwrap_or(0)
            .clamp(0, MAX_SIZE_PER_AXIS);
        let rotation = rotation_to_legacy_name(rotation_from_name(
            nbt.get_string("rotation").unwrap_or("NONE"),
        ))
        .to_string();
        let mirror =
            mirror_to_legacy_name(mirror_from_name(nbt.get_string("mirror").unwrap_or("NONE")))
                .to_string();
        let mode = mode_to_legacy_name(
            mode_from_name(nbt.get_string("mode").unwrap_or("DATA"))
                .unwrap_or(StructureblockMode::Data),
        )
        .to_string();

        Self {
            position,
            name: Mutex::new(canonical_name),
            author: Mutex::new(nbt.get_string("author").unwrap_or("").to_string()),
            metadata: Mutex::new(nbt.get_string("metadata").unwrap_or("").to_string()),
            pos_x: Mutex::new(pos_x),
            pos_y: Mutex::new(pos_y),
            pos_z: Mutex::new(pos_z),
            size_x: Mutex::new(size_x),
            size_y: Mutex::new(size_y),
            size_z: Mutex::new(size_z),
            rotation: Mutex::new(rotation),
            mirror: Mutex::new(mirror),
            mode: Mutex::new(mode),
            ignore_entities: Mutex::new(nbt.get_bool("ignoreEntities").unwrap_or(true)),
            strict: Mutex::new(nbt.get_bool("strict").unwrap_or(false)),
            powered: Mutex::new(nbt.get_bool("powered").unwrap_or(false)),
            show_air: Mutex::new(
                nbt.get_bool("showair")
                    .or_else(|| nbt.get_bool("showAir"))
                    .unwrap_or(false),
            ),
            show_bounding_box: Mutex::new(
                nbt.get_bool("showboundingbox")
                    .or_else(|| nbt.get_bool("showBoundingBox"))
                    .unwrap_or(true),
            ),
            integrity: Mutex::new(nbt.get_float("integrity").unwrap_or(1.0)),
            seed: Mutex::new(nbt.get_long("seed").unwrap_or(0)),
            dirty: AtomicBool::new(false),
        }
    }

    fn write_nbt(&self, nbt: &mut NbtCompound) {
        if let Ok(name) = self.name.lock() {
            nbt.put_string("name", name.clone());
        }
        if let Ok(author) = self.author.lock() {
            nbt.put_string("author", author.clone());
        }
        if let Ok(metadata) = self.metadata.lock() {
            nbt.put_string("metadata", metadata.clone());
        }
        if let Ok(pos_x) = self.pos_x.lock() {
            nbt.put_int("posX", *pos_x);
        }
        if let Ok(pos_y) = self.pos_y.lock() {
            nbt.put_int("posY", *pos_y);
        }
        if let Ok(pos_z) = self.pos_z.lock() {
            nbt.put_int("posZ", *pos_z);
        }
        if let Ok(size_x) = self.size_x.lock() {
            nbt.put_int("sizeX", *size_x);
        }
        if let Ok(size_y) = self.size_y.lock() {
            nbt.put_int("sizeY", *size_y);
        }
        if let Ok(size_z) = self.size_z.lock() {
            nbt.put_int("sizeZ", *size_z);
        }
        if let Ok(rotation) = self.rotation.lock() {
            nbt.put_string("rotation", rotation.clone());
        }
        if let Ok(mirror) = self.mirror.lock() {
            nbt.put_string("mirror", mirror.clone());
        }
        if let Ok(mode) = self.mode.lock() {
            nbt.put_string("mode", mode.clone());
        }
        if let Ok(ignore_entities) = self.ignore_entities.lock() {
            nbt.put_bool("ignoreEntities", *ignore_entities);
        }
        if let Ok(strict) = self.strict.lock() {
            nbt.put_bool("strict", *strict);
        }
        if let Ok(powered) = self.powered.lock() {
            nbt.put_bool("powered", *powered);
        }
        if let Ok(show_air) = self.show_air.lock() {
            nbt.put_bool("showair", *show_air);
        }
        if let Ok(show_bounding_box) = self.show_bounding_box.lock() {
            nbt.put_bool("showboundingbox", *show_bounding_box);
        }
        if let Ok(integrity) = self.integrity.lock() {
            nbt.put_float("integrity", *integrity);
        }
        if let Ok(seed) = self.seed.lock() {
            nbt.put_long("seed", *seed);
        }
    }

    fn chunk_data_nbt(&self) -> Option<NbtCompound> {
        // Blocking lock like the getters: transient contention must not drop the whole payload.
        let mut nbt = NbtCompound::new();
        nbt.put_string(
            "name",
            self.name
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .clone(),
        );
        nbt.put_string(
            "author",
            self.author
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .clone(),
        );
        nbt.put_string(
            "metadata",
            self.metadata
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .clone(),
        );
        nbt.put_int(
            "posX",
            *self
                .pos_x
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner),
        );
        nbt.put_int(
            "posY",
            *self
                .pos_y
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner),
        );
        nbt.put_int(
            "posZ",
            *self
                .pos_z
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner),
        );
        nbt.put_int(
            "sizeX",
            *self
                .size_x
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner),
        );
        nbt.put_int(
            "sizeY",
            *self
                .size_y
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner),
        );
        nbt.put_int(
            "sizeZ",
            *self
                .size_z
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner),
        );
        nbt.put_string(
            "rotation",
            self.rotation
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .clone(),
        );
        nbt.put_string(
            "mirror",
            self.mirror
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .clone(),
        );
        nbt.put_string(
            "mode",
            self.mode
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .clone(),
        );
        nbt.put_bool(
            "ignoreEntities",
            *self
                .ignore_entities
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner),
        );
        nbt.put_bool(
            "strict",
            *self
                .strict
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner),
        );
        nbt.put_bool(
            "powered",
            *self
                .powered
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner),
        );
        nbt.put_bool(
            "showair",
            *self
                .show_air
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner),
        );
        nbt.put_bool(
            "showboundingbox",
            *self
                .show_bounding_box
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner),
        );
        nbt.put_float(
            "integrity",
            *self
                .integrity
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner),
        );
        nbt.put_long(
            "seed",
            *self
                .seed
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner),
        );
        Some(nbt)
    }

    fn is_dirty(&self) -> bool {
        self.dirty.load(Ordering::Relaxed)
    }

    fn clear_dirty(&self) {
        self.dirty.store(false, Ordering::Relaxed);
    }

    fn on_nbt_loaded(&self, world: &Arc<World>, _previous: Option<&dyn BlockEntity>) {
        self.sync_block_state_mode(world);
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl StructureBlockBlockEntity {
    pub const ID: &'static str = "minecraft:structure_block";

    #[must_use]
    pub fn new(position: BlockPos) -> Self {
        Self::new_with_mode(position, StructureblockMode::Load)
    }

    #[must_use]
    pub fn new_with_mode(position: BlockPos, mode: StructureblockMode) -> Self {
        Self {
            position,
            name: Mutex::new(String::new()),
            author: Mutex::new(String::new()),
            metadata: Mutex::new(String::new()),
            pos_x: Mutex::new(DEFAULT_POS_X),
            pos_y: Mutex::new(DEFAULT_POS_Y),
            pos_z: Mutex::new(DEFAULT_POS_Z),
            size_x: Mutex::new(0),
            size_y: Mutex::new(0),
            size_z: Mutex::new(0),
            rotation: Mutex::new("NONE".to_string()),
            mirror: Mutex::new("NONE".to_string()),
            mode: Mutex::new(mode_to_legacy_name(mode).to_string()),
            ignore_entities: Mutex::new(true),
            strict: Mutex::new(false),
            powered: Mutex::new(false),
            show_air: Mutex::new(false),
            show_bounding_box: Mutex::new(true),
            integrity: Mutex::new(1.0),
            seed: Mutex::new(0),
            dirty: AtomicBool::new(false),
        }
    }

    #[must_use]
    pub fn mode_from_block_state(state_id: BlockStateId) -> Option<StructureblockMode> {
        if Block::from_state_id(state_id) != &Block::STRUCTURE_BLOCK {
            return None;
        }
        Some(StructureBlockLikeProperties::from_state_id(state_id).mode)
    }

    #[must_use]
    pub fn mode(&self) -> StructureblockMode {
        let raw = self
            .mode
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        mode_from_name(&raw).unwrap_or(StructureblockMode::Data)
    }

    pub fn set_mode(&self, world: &Arc<World>, mode: StructureblockMode) {
        *self
            .mode
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) =
            mode_to_legacy_name(mode).to_string();
        self.mark_dirty();
        self.sync_block_state_mode(world);
    }

    pub fn sync_block_state_mode(&self, world: &Arc<World>) {
        let mode = self.mode();
        let state_id = world.get_block_state_id(&self.position);
        if Block::from_state_id(state_id) == &Block::STRUCTURE_BLOCK {
            let mut props = StructureBlockLikeProperties::from_state_id(state_id);
            if props.mode != mode {
                props.mode = mode;
                world.set_block_state(
                    &self.position,
                    props.to_state_id(&Block::STRUCTURE_BLOCK),
                    BlockFlags::NOTIFY_LISTENERS,
                );
            }
        }
    }

    pub fn set_author(&self, author: impl Into<String>) {
        *self
            .author
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = author.into();
        self.mark_dirty();
    }

    pub fn set_structure_name(&self, raw_name: &str) {
        let canonical = parse_structure_identifier(raw_name)
            .map(|(_, _, id)| id)
            .unwrap_or_default();
        *self
            .name
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = canonical;
        self.mark_dirty();
    }

    #[must_use]
    pub fn get_structure_name(&self) -> String {
        self.name
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }

    #[must_use]
    pub fn has_structure_name(&self) -> bool {
        !self.get_structure_name().is_empty()
    }

    pub fn mark_dirty(&self) {
        self.dirty.store(true, Ordering::Release);
    }

    #[must_use]
    pub fn is_powered(&self) -> bool {
        *self
            .powered
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    pub fn set_powered(&self, powered: bool) {
        *self
            .powered
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = powered;
        self.mark_dirty();
    }

    #[must_use]
    pub fn structure_pos(&self) -> Vector3<i32> {
        let x = *self
            .pos_x
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let y = *self
            .pos_y
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let z = *self
            .pos_z
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        Vector3::new(x, y, z)
    }

    pub fn set_structure_pos(&self, pos: Vector3<i32>) {
        *self
            .pos_x
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = pos.x;
        *self
            .pos_y
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = pos.y;
        *self
            .pos_z
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = pos.z;
        self.mark_dirty();
    }

    #[must_use]
    pub fn structure_size(&self) -> Vector3<i32> {
        let x = *self
            .size_x
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let y = *self
            .size_y
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let z = *self
            .size_z
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        Vector3::new(x, y, z)
    }

    pub fn set_structure_size(&self, size: Vector3<i32>) {
        *self
            .size_x
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = size.x;
        *self
            .size_y
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = size.y;
        *self
            .size_z
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = size.z;
        self.mark_dirty();
    }

    #[must_use]
    pub fn mirror_value(&self) -> Mirror {
        let raw = self
            .mirror
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        mirror_from_name(&raw)
    }

    pub fn set_mirror_value(&self, mirror: Mirror) {
        *self
            .mirror
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) =
            mirror_to_legacy_name(mirror).to_string();
        self.mark_dirty();
    }

    #[must_use]
    pub fn rotation_value(&self) -> Rotation {
        let raw = self
            .rotation
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        rotation_from_name(&raw)
    }

    pub fn set_rotation_value(&self, rotation: Rotation) {
        *self
            .rotation
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) =
            rotation_to_legacy_name(rotation).to_string();
        self.mark_dirty();
    }

    pub fn detect_size(&self, world: &Arc<World>) -> bool {
        if self.mode() != StructureblockMode::Save {
            return false;
        }
        let target_name = self.get_structure_name();
        if target_name.is_empty() {
            return false;
        }

        let min_x = self.position.0.x - SCAN_CORNER_BLOCKS_RANGE;
        let max_x = self.position.0.x + SCAN_CORNER_BLOCKS_RANGE;
        let min_z = self.position.0.z - SCAN_CORNER_BLOCKS_RANGE;
        let max_z = self.position.0.z + SCAN_CORNER_BLOCKS_RANGE;
        let min_y = world.get_bottom_y();
        let max_y = world.get_top_y();

        let min_chunk_x = min_x >> 4;
        let max_chunk_x = max_x >> 4;
        let min_chunk_z = min_z >> 4;
        let max_chunk_z = max_z >> 4;

        let mut candidate_positions = Vec::new();
        for cx in min_chunk_x..=max_chunk_x {
            for cz in min_chunk_z..=max_chunk_z {
                let chunk_pos = Vector2::new(cx, cz);
                let found = world.level.read_chunk_sync(&chunk_pos, |chunk| {
                    let sections = chunk
                        .section
                        .block_sections
                        .read()
                        .unwrap_or_else(std::sync::PoisonError::into_inner);
                    collect_structure_block_positions(
                        &sections,
                        chunk.x,
                        chunk.z,
                        chunk.section.min_y,
                        min_y,
                        max_y,
                    )
                });
                if let Some(found) = found {
                    candidate_positions.extend(found.into_iter().filter(|pos| {
                        pos.0.x >= min_x && pos.0.x <= max_x && pos.0.z >= min_z && pos.0.z <= max_z
                    }));
                }
            }
        }

        let mut matching_corners = Vec::new();
        for pos in candidate_positions {
            let Some(be) = world.get_block_entity(&pos) else {
                continue;
            };
            let Some(other) = be.as_any().downcast_ref::<Self>() else {
                continue;
            };
            if other.mode() == StructureblockMode::Corner
                && other.get_structure_name() == target_name
            {
                matching_corners.push(pos);
            }
        }

        let Some((offset, size)) = compute_detected_bounds(self.position, &matching_corners) else {
            return false;
        };

        self.set_structure_pos(offset);
        self.set_structure_size(size);
        true
    }

    pub fn save_structure_area(&self, world: &Arc<World>) -> bool {
        if self.mode() != StructureblockMode::Save {
            return false;
        }
        self.save_structure(world, true)
    }

    pub fn save_structure(&self, world: &Arc<World>, save_to_disk: bool) -> bool {
        let structure_name = self.get_structure_name();
        if structure_name.is_empty() {
            return false;
        }
        let offset = self.structure_pos();
        let origin = BlockPos::new(
            self.position.0.x + offset.x,
            self.position.0.y + offset.y,
            self.position.0.z + offset.z,
        );
        let size = self.structure_size();
        let author = self
            .author
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone();
        let include_entities = !*self
            .ignore_entities
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);

        save_structure_from_world(
            world,
            &structure_name,
            origin,
            size,
            &author,
            save_to_disk,
            &[],
            include_entities,
        )
    }

    #[must_use]
    pub fn is_structure_loadable(&self, world: &World) -> bool {
        if self.mode() != StructureblockMode::Load {
            return false;
        }
        let structure_name = self.get_structure_name();
        if structure_name.is_empty() {
            return false;
        }
        load_or_cache_structure_template(world, &structure_name).is_some()
    }

    fn load_structure_info(&self, template: &StructureTemplate) {
        let author = template.get_author();
        self.set_author(if author.is_empty() { "" } else { author });
        self.set_structure_size(template.get_size());
    }

    pub fn place_structure_if_same_size(&self, world: &Arc<World>) -> bool {
        if self.mode() != StructureblockMode::Load {
            return false;
        }
        let structure_name = self.get_structure_name();
        if structure_name.is_empty() {
            return false;
        }
        let Some(template) = load_or_cache_structure_template(world, &structure_name) else {
            return false;
        };
        if template.get_size() == self.structure_size() {
            self.place_loaded_template(world, &template);
            true
        } else {
            self.load_structure_info(&template);
            false
        }
    }

    pub fn place_structure(&self, world: &Arc<World>) {
        let structure_name = self.get_structure_name();
        if structure_name.is_empty() {
            return;
        }
        if let Some(template) = load_or_cache_structure_template(world, &structure_name) {
            self.place_loaded_template(world, &template);
        }
    }

    fn place_loaded_template(&self, world: &Arc<World>, template: &StructureTemplate) {
        self.load_structure_info(template);
        let offset = self.structure_pos();
        let origin = BlockPos::new(
            self.position.0.x + offset.x,
            self.position.0.y + offset.y,
            self.position.0.z + offset.z,
        );
        let mirror = self.mirror_value();
        let rotation = self.rotation_value();
        let integrity = *self
            .integrity
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let seed = *self
            .seed
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let strict = *self
            .strict
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let ignore_entities = *self
            .ignore_entities
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);

        place_structure_template_in_world(
            world,
            template,
            origin,
            &StructurePlacementSettings {
                mirror,
                rotation,
                integrity,
                seed,
                strict,
                ignore_entities,
            },
        );
    }

    pub fn unload_structure(&self) {
        let structure_name = self.get_structure_name();
        if !structure_name.is_empty() {
            global_cache().remove_dynamic(&structure_name);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        NbtCompound, NbtTag, collect_structure_block_positions, compute_detected_bounds,
        edge_neighbor, is_randomizable_container_id, is_safe_generated_structure_path,
        parse_structure_identifier, put_transformed_block_pos, transform_entity_yaw,
    };
    use pumpkin_data::{Block, BlockDirection, Mirror, Rotation};
    use pumpkin_util::math::{position::BlockPos, vector3::Vector3, wrap_degrees};
    use std::collections::HashSet;

    #[test]
    fn dynamic_shape_blocks_are_not_full_cube_bucketed() {
        assert!(super::has_dynamic_shape("moving_piston"));
        assert!(super::has_dynamic_shape("bamboo"));
        assert!(super::has_dynamic_shape("shulker_box"));
        assert!(super::has_dynamic_shape("red_shulker_box"));
        assert!(!super::has_dynamic_shape("stone"));
        assert!(!super::has_dynamic_shape("glass"));
        assert!(!super::has_dynamic_shape("bamboo_sapling"));
    }

    #[test]
    fn palette_full_block_uses_collision_shape_not_occlusion() {
        // Glass has a full collision shape but does not occlude, so it must sort as a full block.
        let glass = Block::GLASS.default_state;
        assert!(glass.is_full_cube());
        assert!(!glass.is_solid_render());
        assert!(!Block::WATER.default_state.is_full_cube());
    }

    #[test]
    fn loot_seed_only_for_randomizable_containers() {
        assert!(is_randomizable_container_id("minecraft:chest"));
        assert!(is_randomizable_container_id("minecraft:red_shulker_box"));
        assert!(is_randomizable_container_id("minecraft:dropper"));
        assert!(!is_randomizable_container_id("minecraft:brewing_stand"));
        assert!(!is_randomizable_container_id("minecraft:furnace"));
        assert!(!is_randomizable_container_id("minecraft:sign"));
    }

    #[test]
    fn structure_identifier_validation_canonicalizes_and_rejects_invalid_names() {
        assert_eq!(
            parse_structure_identifier("house/roof"),
            Some((
                "minecraft".to_string(),
                "house/roof".to_string(),
                "minecraft:house/roof".to_string()
            ))
        );
        assert_eq!(
            parse_structure_identifier("custom_ns:sub/part_1"),
            Some((
                "custom_ns".to_string(),
                "sub/part_1".to_string(),
                "custom_ns:sub/part_1".to_string()
            ))
        );
        assert_eq!(parse_structure_identifier(""), None);
        assert_eq!(parse_structure_identifier("InvalidUpper:test"), None);
        assert_eq!(parse_structure_identifier("minecraft:bad space"), None);
        assert_eq!(parse_structure_identifier(":missing_ns"), None);
        assert_eq!(parse_structure_identifier("missing_path:"), None);

        assert!(is_safe_generated_structure_path("house/roof"));
        assert!(!is_safe_generated_structure_path("../escape"));
        assert!(!is_safe_generated_structure_path("a/../b"));
        assert!(!is_safe_generated_structure_path("/leading"));
        assert!(!is_safe_generated_structure_path("trailing/"));
    }

    #[test]
    fn detect_size_matches_single_and_multi_corner_vanilla_rules() {
        let save_pos = BlockPos::new(10, 64, 10);

        // Single corner encapsulates both save_pos and the corner, then shrinks inward by 1.
        let (offset, size) =
            compute_detected_bounds(save_pos, &[BlockPos::new(15, 69, 14)]).unwrap();
        assert_eq!(offset, Vector3::new(1, 1, 1));
        assert_eq!(size, Vector3::new(4, 4, 3));

        // Two corners define the outer shell directly without including save_pos.
        let (offset, size) = compute_detected_bounds(
            save_pos,
            &[BlockPos::new(12, 63, 12), BlockPos::new(18, 70, 20)],
        )
        .unwrap();
        assert_eq!(offset, Vector3::new(3, 0, 3));
        assert_eq!(size, Vector3::new(5, 6, 7));

        // Adjacent corners with zero interior volume fail detection.
        assert!(compute_detected_bounds(save_pos, &[BlockPos::new(11, 65, 11)]).is_none());
    }

    #[test]
    fn structure_block_scan_selects_block_states_within_the_scan_box() {
        let mut palette = pumpkin_world::chunk::palette::BlockPalette::default();
        palette.set(1, 2, 3, Block::STRUCTURE_BLOCK.default_state.id);
        palette.set(4, 5, 6, Block::STONE.default_state.id);

        let sections = std::slice::from_ref(&palette);
        let found = collect_structure_block_positions(sections, 2, -1, -64, -64, 319);
        assert_eq!(found, vec![BlockPos(Vector3::new(33, -62, -13))]);

        // Vanilla clamps the scan box to the build height, so a corner above it is skipped.
        assert!(collect_structure_block_positions(sections, 2, -1, -64, -56, 319).is_empty());
    }

    #[test]
    fn transform_entity_yaw_matches_vanilla_rotate_then_mirror() {
        let close = |a: f32, b: f32| (a - b).abs() < 1e-5;
        assert!(close(
            transform_entity_yaw(90.0, Mirror::None, Rotation::None),
            90.0
        ));
        assert!(close(
            transform_entity_yaw(90.0, Mirror::None, Rotation::Rotate180),
            270.0
        ));
        assert!(close(
            transform_entity_yaw(30.0, Mirror::LeftRight, Rotation::None),
            150.0
        ));
        assert!(close(
            transform_entity_yaw(90.0, Mirror::FrontBack, Rotation::None),
            -90.0
        ));
        // Vanilla applies rotate first, then mirror (StructureTemplate#placeEntities).
        assert!(close(
            transform_entity_yaw(30.0, Mirror::FrontBack, Rotation::Clockwise90),
            60.0
        ));
        let wrapped = transform_entity_yaw(370.0, Mirror::None, Rotation::Clockwise90);
        assert!(close(wrap_degrees(wrapped), 100.0));
    }

    #[test]
    fn entity_block_pos_uses_three_int_codec_shape() {
        let mut nbt = NbtCompound::new();
        nbt.put("block_pos", NbtTag::IntArray(vec![0, 0, 0]));
        put_transformed_block_pos(&mut nbt, Vector3::new(3, -4, 5));
        assert_eq!(nbt.get_int_array("block_pos"), Some(&[3, -4, 5][..]));
    }

    #[test]
    fn edge_neighbor_is_only_outside_the_placed_shape() {
        let a = BlockPos::new(0, 0, 0);
        let b = BlockPos::new(1, 0, 0);
        let placed: HashSet<BlockPos> = [a, b].into_iter().collect();

        // b is placed, so a's +X face is interior and vanilla skips it.
        assert_eq!(edge_neighbor(&placed, &a, BlockDirection::East), None);
        // -X of a and +X of b are outside the shape: both are boundary faces.
        assert_eq!(
            edge_neighbor(&placed, &a, BlockDirection::West),
            Some(BlockPos::new(-1, 0, 0))
        );
        assert_eq!(
            edge_neighbor(&placed, &b, BlockDirection::East),
            Some(BlockPos::new(2, 0, 0))
        );
    }
}
