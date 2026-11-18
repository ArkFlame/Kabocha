use super::BlockEntity;
use crate::entity::EntityBase;
use crate::entity::r#type::load_entity_from_nbt;
use crate::world::World;
use pumpkin_data::block_properties::Facing;
use pumpkin_data::{Property, PropertyKey};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_nbt::tag::NbtTag;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use std::sync::{Arc, Mutex, PoisonError};

pub struct BeehiveBlockEntity {
    pub position: BlockPos,
    pub bees: Mutex<Option<Vec<NbtTag>>>,
    pub flower_pos: Mutex<Option<BlockPos>>,
}

impl BlockEntity for BeehiveBlockEntity {
    fn resource_location(&self) -> &'static str {
        Self::ID
    }

    fn get_position(&self) -> BlockPos {
        self.position
    }

    fn from_nbt(nbt: &pumpkin_nbt::compound::NbtCompound, position: BlockPos) -> Self
    where
        Self: Sized,
    {
        let bees = nbt
            .get_list("bees")
            .or_else(|| nbt.get_list("Bees"))
            .map(<[_]>::to_vec);
        let flower_pos = nbt
            .get_int_array("flower_pos")
            .and_then(|arr| (arr.len() == 3).then(|| BlockPos::new(arr[0], arr[1], arr[2])))
            .or_else(|| {
                nbt.get_compound("flower_pos")
                    .or_else(|| nbt.get_compound("FlowerPos"))
                    .map(|c| {
                        BlockPos::new(
                            c.get_int("X").or_else(|| c.get_int("x")).unwrap_or(0),
                            c.get_int("Y").or_else(|| c.get_int("y")).unwrap_or(0),
                            c.get_int("Z").or_else(|| c.get_int("z")).unwrap_or(0),
                        )
                    })
            });
        Self {
            position,
            bees: Mutex::new(bees),
            flower_pos: Mutex::new(flower_pos),
        }
    }

    fn write_nbt(&self, nbt: &mut NbtCompound) {
        if let Ok(b) = self.bees.lock()
            && let Some(b) = b.as_ref()
        {
            nbt.put_list("bees", b.clone());
        }
        if let Ok(fp) = self.flower_pos.lock()
            && let Some(fp) = fp.as_ref()
        {
            nbt.put(
                "flower_pos",
                pumpkin_nbt::tag::NbtTag::IntArray(vec![fp.0.x, fp.0.y, fp.0.z]),
            );
        }
    }

    fn tick(&self, world: &Arc<World>) {
        self.tick_occupants(world);
    }

    fn chunk_data_nbt(&self) -> Option<NbtCompound> {
        let mut nbt = NbtCompound::new();
        if let Ok(bees) = self.bees.try_lock()
            && let Some(ref b) = *bees
        {
            nbt.put_list("bees", b.clone());
        }
        if let Ok(flower_pos) = self.flower_pos.try_lock()
            && let Some(ref fp) = *flower_pos
        {
            nbt.put(
                "flower_pos",
                pumpkin_nbt::tag::NbtTag::IntArray(vec![fp.0.x, fp.0.y, fp.0.z]),
            );
        }
        Some(nbt)
    }

    fn collect_item_components(&self, stack: &mut pumpkin_data::item_stack::ItemStack) {
        if let Ok(bees) = self.bees.lock()
            && let Some(bees) = bees.as_ref()
            && !bees.is_empty()
        {
            stack.set_data_component(pumpkin_data::data_component_impl::BeesImpl {
                bees: bees.clone(),
            });
        }
    }

    fn apply_item_components(&self, stack: &pumpkin_data::item_stack::ItemStack) {
        if let Some(bees) =
            stack.get_data_component::<pumpkin_data::data_component_impl::BeesImpl>()
            && !bees.bees.is_empty()
        {
            *self
                .bees
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(bees.bees.clone());
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Vanilla `BeehiveBlockEntity` front-face offset from its facing block property.
const fn front_offset(facing: Option<Property>) -> (f64, f64) {
    match facing {
        Some(Property::Facing(Facing::North)) => (0.0, -1.0),
        Some(Property::Facing(Facing::East)) => (1.0, 0.0),
        Some(Property::Facing(Facing::West)) => (-1.0, 0.0),
        _ => (0.0, 1.0),
    }
}

/// Reads a vanilla `BlockPos.CODEC` compound (`X`/`Y`/`Z`, or the legacy lowercase form).
fn position_from_compound(compound: &NbtCompound) -> Option<BlockPos> {
    let x = compound.get_int("X").or_else(|| compound.get_int("x"))?;
    let y = compound.get_int("Y").or_else(|| compound.get_int("y"))?;
    let z = compound.get_int("Z").or_else(|| compound.get_int("z"))?;
    Some(BlockPos::new(x, y, z))
}

/// Vanilla `BeehiveBlockEntity.addOccupant`: a hive with no remembered flower adopts the
/// bee's, otherwise a coin flip decides.
const fn adopts_flower(hive_has_flower: bool, coin_flip: bool) -> bool {
    !hive_has_flower || coin_flip
}

/// Vanilla `BeehiveBlockEntity.releaseOccupant`: a released bee inherits the hive's flower
/// only when it has none and the 90% roll passed (`roll_passed`).
const fn inherits_flower(bee_has_flower: bool, hive_has_flower: bool, roll_passed: bool) -> bool {
    hive_has_flower && !bee_has_flower && roll_passed
}

impl BeehiveBlockEntity {
    pub const ID: &'static str = "minecraft:beehive";
    /// Vanilla `BeehiveBlockEntity.MAX_OCCUPANTS`.
    pub const MAX_OCCUPANTS: usize = 3;
    const MIN_TICKS_IN_HIVE: i32 = 600;
    const NECTAR_TICKS_IN_HIVE: i32 = 2400;

    #[must_use]
    pub const fn new(position: BlockPos) -> Self {
        Self {
            position,
            bees: Mutex::new(None),
            flower_pos: Mutex::new(None),
        }
    }

    #[must_use]
    pub fn is_full(&self) -> bool {
        self.bees
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .as_ref()
            .is_some_and(|bees| bees.len() >= Self::MAX_OCCUPANTS)
    }

    /// Vanilla BeehiveBlockEntity.addOccupant: persist the bee's entity data and remove the
    /// live entity. Returns whether an occupant slot was available.
    pub fn add_occupant(&self, bee: &dyn EntityBase) -> bool {
        let mut entity_data = NbtCompound::new();
        EntityBase::write_nbt(bee, &mut entity_data);
        let has_nectar = entity_data.get_bool("HasNectar").unwrap_or(false);
        let bee_flower = entity_data
            .get_compound("flower_pos")
            .and_then(position_from_compound);
        let mut guard = self.bees.lock().unwrap_or_else(PoisonError::into_inner);
        let bees = guard.get_or_insert_with(Vec::new);
        if bees.len() >= Self::MAX_OCCUPANTS {
            return false;
        }
        let mut occupant = NbtCompound::new();
        occupant.put_compound("entity_data", entity_data);
        occupant.put_int("ticks_in_hive", 0);
        // Vanilla Occupant.of: nectar-carrying bees stay inside for 2400 ticks, others 600.
        occupant.put_int(
            "min_ticks_in_hive",
            if has_nectar {
                Self::NECTAR_TICKS_IN_HIVE
            } else {
                Self::MIN_TICKS_IN_HIVE
            },
        );
        bees.push(NbtTag::Compound(occupant));
        // Vanilla BeehiveBlockEntity.addOccupant stores the bee's remembered flower on the hive.
        if let Some(flower) = bee_flower {
            let mut hive_flower = self
                .flower_pos
                .lock()
                .unwrap_or_else(PoisonError::into_inner);
            if adopts_flower(hive_flower.is_some(), rand::random::<bool>()) {
                *hive_flower = Some(flower);
            }
        }
        true
    }

    /// Vanilla BeehiveBlockEntity.tickOccupants: age stored bees and respawn the finished ones
    /// in front of the hive. Hiving without this release would permanently delete the bee.
    pub fn tick_occupants(&self, world: &Arc<World>) {
        let ready: Vec<NbtCompound> = {
            let mut guard = self.bees.lock().unwrap_or_else(PoisonError::into_inner);
            let Some(bees) = guard.as_mut() else {
                return;
            };
            let mut ready = Vec::new();
            for bee in bees.iter_mut() {
                let NbtTag::Compound(compound) = bee else {
                    continue;
                };
                let ticks = compound.get_int("ticks_in_hive").unwrap_or(0) + 1;
                compound.put_int("ticks_in_hive", ticks);
                let min = compound
                    .get_int("min_ticks_in_hive")
                    .unwrap_or(Self::MIN_TICKS_IN_HIVE);
                if ticks >= min
                    && let Some(entity_data) = compound.get_compound("entity_data").cloned()
                {
                    ready.push(entity_data);
                }
            }
            bees.retain(|bee| match bee {
                NbtTag::Compound(c) => {
                    c.get_int("ticks_in_hive").unwrap_or(0)
                        < c.get_int("min_ticks_in_hive")
                            .unwrap_or(Self::MIN_TICKS_IN_HIVE)
                }
                _ => true,
            });
            ready
        };
        if ready.is_empty() {
            return;
        }
        let hive_flower = *self
            .flower_pos
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        let state_id = world.get_block_state_id(&self.position);
        let facing = world
            .get_block(&self.position)
            .get_property(state_id, PropertyKey::Facing);
        let (dx, dz) = front_offset(facing);
        let pos = self.position;
        let spawn = Vector3::new(
            f64::from(pos.0.x) + 0.5 + 0.6 * dx,
            f64::from(pos.0.y) + 0.5 - 0.4,
            f64::from(pos.0.z) + 0.5 + 0.6 * dz,
        );
        for mut entity_data in ready {
            // Vanilla HONEY_DELIVERED release calls Bee.dropOffNectar, which resets the
            // no-nectar timer; without this a released forager would re-enter immediately.
            if entity_data.get_bool("HasNectar").unwrap_or(false) {
                entity_data.put_bool("HasNectar", false);
                entity_data.put_int("TicksSincePollination", 0);
                entity_data.put_int("CropsGrownSincePollination", 0);
            }
            // Vanilla BeehiveBlockEntity.releaseOccupant gives a bee with no remembered
            // flower the hive's flower with 90% chance.
            let bee_has_flower = entity_data.get_compound("flower_pos").is_some()
                || entity_data.get_int_array("flower_pos").is_some();
            let roll_passed = rand::random::<f32>() < 0.9f32;
            if inherits_flower(bee_has_flower, hive_flower.is_some(), roll_passed)
                && let Some(flower) = hive_flower
            {
                let mut tag = NbtCompound::new();
                tag.put_int("X", flower.0.x);
                tag.put_int("Y", flower.0.y);
                tag.put_int("Z", flower.0.z);
                entity_data.put_compound("flower_pos", tag);
            }
            let Some(entity) = load_entity_from_nbt(&entity_data, spawn, world) else {
                continue;
            };
            // Entity.read_placement_nbt restores the saved Pos, so override it with the
            // front-face spawn, like vanilla Occupant.createEntity.
            entity.get_entity().set_pos(spawn);
            entity.get_entity().set_has_no_gravity(true);
            world.spawn_entity(entity);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn releases_from_the_front_face() {
        assert_eq!(
            front_offset(Some(Property::Facing(Facing::North))),
            (0.0, -1.0)
        );
        assert_eq!(
            front_offset(Some(Property::Facing(Facing::East))),
            (1.0, 0.0)
        );
        assert_eq!(
            front_offset(Some(Property::Facing(Facing::South))),
            (0.0, 1.0)
        );
        assert_eq!(
            front_offset(Some(Property::Facing(Facing::West))),
            (-1.0, 0.0)
        );
        assert_eq!(front_offset(None), (0.0, 1.0));
    }

    #[test]
    fn hive_adopts_flower_with_coin_flip_rules() {
        assert!(adopts_flower(false, false));
        assert!(adopts_flower(false, true));
        assert!(adopts_flower(true, true));
        assert!(!adopts_flower(true, false));
    }

    #[test]
    fn released_bee_inherits_flower_only_when_eligible() {
        assert!(inherits_flower(false, true, true));
        assert!(!inherits_flower(true, true, true));
        assert!(!inherits_flower(false, false, true));
        assert!(!inherits_flower(false, true, false));
    }

    #[test]
    fn reads_flower_position_compound() {
        let mut tag = NbtCompound::new();
        tag.put_int("X", 1);
        tag.put_int("Y", 2);
        tag.put_int("Z", 3);
        assert_eq!(position_from_compound(&tag), Some(BlockPos::new(1, 2, 3)));
        let mut partial = NbtCompound::new();
        partial.put_int("X", 1);
        assert_eq!(position_from_compound(&partial), None);
    }
}
