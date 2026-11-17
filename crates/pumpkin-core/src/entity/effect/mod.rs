pub mod hunger;
pub mod infested;
pub mod oozing;
pub mod poison;
pub mod raid_omen;
pub mod regeneration;
pub mod saturation;
pub mod weaving;
pub mod wind_charged;
pub mod wither;

use pumpkin_data::damage::DamageType;
use pumpkin_data::effect::StatusEffect;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_nbt::tag::NbtTag;
use tracing::warn;

use crate::entity::living::LivingEntity;
use crate::entity::{NBTStorage, NBTStorageInit};

pub trait MobEffect: Send + Sync {
    /// Returns true if `apply_effect_tick` should be called for the current tick and duration.
    fn should_apply_effect_tick(&self, _duration: i32, _amplifier: u8) -> bool {
        false
    }

    /// Applies periodic/tick-based effect logic on a living entity.
    fn apply_effect_tick(&self, _living: &LivingEntity, _amplifier: u8) {}

    /// Called when an entity carrying this effect is hurt.
    fn on_mob_hurt(
        &self,
        _living: &LivingEntity,
        _amplifier: u8,
        _damage_type: &DamageType,
        _damage_amount: f32,
    ) {
    }

    /// Called when an entity carrying this effect dies from damage.
    fn on_mob_death(&self, living: &LivingEntity, amplifier: u8, _damage_type: &DamageType) {
        self.on_mob_removed(living, amplifier);
    }

    /// Called when a killed entity carrying this effect is removed, vanilla `onMobRemoved`.
    /// Needs no damage source, so it also fires when a creeper discards itself in an explosion.
    fn on_mob_removed(&self, _living: &LivingEntity, _amplifier: u8) {}
}

pub static REGENERATION: regeneration::RegenerationMobEffect = regeneration::RegenerationMobEffect;
pub static POISON: poison::PoisonMobEffect = poison::PoisonMobEffect;
pub static WITHER: wither::WitherMobEffect = wither::WitherMobEffect;
pub static HUNGER: hunger::HungerMobEffect = hunger::HungerMobEffect;
pub static SATURATION: saturation::SaturationMobEffect = saturation::SaturationMobEffect;
pub static RAID_OMEN: raid_omen::RaidOmenMobEffect = raid_omen::RaidOmenMobEffect;
pub static INFESTED: infested::InfestedMobEffect = infested::InfestedMobEffect;
pub static OOZING: oozing::OozingMobEffect = oozing::OozingMobEffect;
pub static WEAVING: weaving::WeavingMobEffect = weaving::WeavingMobEffect;
pub static WIND_CHARGED: wind_charged::WindChargedMobEffect = wind_charged::WindChargedMobEffect;

/// Vanilla `tick % (base >> amplifier) == 0`, with an interval of 0 or less applying every tick.
/// `wrapping_shr` masks the shift to 5 bits like Java, so high amplifiers wrap the same way.
#[must_use]
pub fn is_interval_tick(tick: i32, base: i32, amplifier: u8) -> bool {
    match base.wrapping_shr(u32::from(amplifier)) {
        interval if interval > 0 => tick % interval == 0,
        _ => true,
    }
}

#[must_use]
pub fn get_mob_effect(effect: &'static StatusEffect) -> Option<&'static dyn MobEffect> {
    if effect == &StatusEffect::REGENERATION {
        Some(&REGENERATION)
    } else if effect == &StatusEffect::POISON {
        Some(&POISON)
    } else if effect == &StatusEffect::WITHER {
        Some(&WITHER)
    } else if effect == &StatusEffect::HUNGER {
        Some(&HUNGER)
    } else if effect == &StatusEffect::SATURATION {
        Some(&SATURATION)
    } else if effect == &StatusEffect::RAID_OMEN {
        Some(&RAID_OMEN)
    } else if effect == &StatusEffect::INFESTED {
        Some(&INFESTED)
    } else if effect == &StatusEffect::OOZING {
        Some(&OOZING)
    } else if effect == &StatusEffect::WEAVING {
        Some(&WEAVING)
    } else if effect == &StatusEffect::WIND_CHARGED {
        Some(&WIND_CHARGED)
    } else {
        None
    }
}

impl NBTStorage for pumpkin_data::potion::Effect {
    fn write_nbt(&self, nbt: &mut NbtCompound) {
        nbt.put("id", self.effect_type.minecraft_name);
        if self.amplifier > 0 {
            nbt.put("amplifier", NbtTag::Int(i32::from(self.amplifier)));
        }
        nbt.put("duration", NbtTag::Int(self.duration));
        if self.ambient {
            nbt.put("ambient", NbtTag::Byte(1));
        }
        if !self.show_particles {
            nbt.put("show_particles", NbtTag::Byte(0));
        }
        let show_icon: i8 = i8::from(self.show_icon);
        nbt.put("show_icon", NbtTag::Byte(show_icon));
    }
}

impl NBTStorageInit for pumpkin_data::potion::Effect {
    fn create_from_nbt(nbt: &mut NbtCompound) -> Option<Self> {
        let Some(effect_id) = nbt.get_string("id") else {
            warn!("Unable to read effect. Effect id is not present");
            return None;
        };
        let Some(effect_type) = StatusEffect::from_minecraft_name(effect_id) else {
            warn!("Unable to read effect. Unknown effect type: {effect_id}");
            return None;
        };
        // Vanilla MobEffectInstance.load defaults show_icon to true when the key is absent.
        let show_icon = nbt.get_byte("show_icon").is_none_or(|b| b == 1);
        let amplifier = nbt.get_int("amplifier").unwrap_or(0) as u8;
        let duration = nbt.get_int("duration").unwrap_or(0);
        let ambient = nbt.get_byte("ambient").unwrap_or(0) == 1;
        let show_particles = nbt.get_byte("show_particles").unwrap_or(1) == 1;
        Some(Self {
            effect_type,
            duration,
            amplifier,
            ambient,
            show_particles,
            show_icon,
            blend: false,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_from_nbt_defaults_show_icon_to_true() {
        let mut nbt = NbtCompound::new();
        nbt.put("id", StatusEffect::REGENERATION.minecraft_name);
        nbt.put("duration", NbtTag::Int(200));

        let effect = <pumpkin_data::potion::Effect as NBTStorageInit>::create_from_nbt(&mut nbt);

        assert!(effect.is_some_and(|effect| effect.show_icon));
    }

    #[test]
    fn interval_tick_follows_vanilla_shift() {
        assert!(is_interval_tick(0, 50, 0));
        assert!(!is_interval_tick(25, 50, 0));
        assert!(is_interval_tick(25, 50, 1));
        assert!(is_interval_tick(3, 50, 4));
        assert!(!is_interval_tick(2, 50, 4));
        assert!(is_interval_tick(3, 50, 5));
        assert!(is_interval_tick(7, 50, 6));
    }
}
