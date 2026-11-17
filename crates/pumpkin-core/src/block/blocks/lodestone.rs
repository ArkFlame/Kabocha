use pumpkin_macros::pumpkin_block;

use crate::block::BlockBehaviour;

#[pumpkin_block("minecraft:lodestone")]
pub struct LodestoneBlock;

// Vanilla registers lodestone as a plain Block with no overrides. POI add/remove is derived
// generically from block state ids by World::update_poi (state 25187 maps to PoiType::Lodestone),
// so the block needs no hooks of its own.
impl BlockBehaviour for LodestoneBlock {}
