use std::any::Any;

use crate::entity::player::Player;
use crate::item::{ItemBehaviour, ItemMetadata};
use crate::world::game_event::GameEventContext;
use pumpkin_data::data_component_impl::InstrumentImpl;
use pumpkin_data::game_event::GameEvent;
use pumpkin_data::instrument::Instrument;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::SoundCategory;
use pumpkin_data::statistic::StatisticCategory;
use pumpkin_util::Hand;

pub struct GoatHornItem;

impl ItemMetadata for GoatHornItem {
    fn ids() -> Box<[u16]> {
        Box::new([Item::GOAT_HORN.id])
    }
}

impl ItemBehaviour for GoatHornItem {
    fn normal_use(&self, item: &Item, player: &Player, hand: Hand) {
        let (yaw, pitch) = player.rotation();
        self.normal_use_with_rotation(item, player, yaw, pitch, hand);
    }

    fn normal_use_with_rotation(
        &self,
        item: &Item,
        player: &Player,
        _yaw: f32,
        _pitch: f32,
        hand: Hand,
    ) {
        let stack = player.inventory().get_stack_in_hand(hand);
        let Some(instrument) = stack
            .get_data_component::<InstrumentImpl>()
            .and_then(|inst| Instrument::from_name(&inst.instrument))
        else {
            return;
        };

        let duration = i32::try_from(instrument.use_duration_ticks()).unwrap_or(i32::MAX);
        let volume = instrument.range() / 16.0;
        let world = player.world();
        let pos = player.position();

        let cooldown_group = stack
            .get_use_cooldown()
            .and_then(|c| c.cooldown_group.clone())
            .unwrap_or_else(|| item.registry_key.to_string());
        if player.is_on_cooldown(&cooldown_group) {
            return;
        }

        player.living_entity.set_active_hand(hand, stack, duration);
        world.play_sound_raw(
            instrument.sound() as u16,
            SoundCategory::Records,
            &pos,
            volume,
            1.0,
        );
        // Vanilla passes `GameEvent.Context.of(player)`, so listeners attribute the
        // vibration to the horn player (e.g. sculk sensor / warden anger).
        world.emit_game_event_with_context(
            GameEvent::InstrumentPlay.name(),
            pos,
            &GameEventContext {
                source_entity: Some(player),
            },
        );
        // Vanilla only adds the cooldown when the instrument has a positive use duration
        // (InstrumentItem.use: `if (instrumentCooldown > 0.0F)`).
        if duration > 0 {
            player.start_cooldown(cooldown_group, duration);
        }
        player.increment_stat(StatisticCategory::Used, i32::from(item.id), 1);
    }

    fn get_use_duration(&self, stack: &ItemStack) -> i32 {
        stack
            .get_data_component::<InstrumentImpl>()
            .and_then(|inst| Instrument::from_name(&inst.instrument))
            .map_or(0, |inst| {
                i32::try_from(inst.use_duration_ticks()).unwrap_or(i32::MAX)
            })
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl GoatHornItem {
    pub const USE_DURATION: i32 = Instrument::PonderGoatHorn.use_duration_ticks() as i32;
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use pumpkin_data::data_component_impl::InstrumentImpl;
    use pumpkin_data::item_stack::ItemStack;

    use super::{GoatHornItem, Instrument, Item, ItemBehaviour};

    #[test]
    fn use_duration_is_read_from_the_stack_instrument() {
        let mut stack = ItemStack::new(1, &Item::GOAT_HORN);
        assert_eq!(GoatHornItem.get_use_duration(&stack), 0);

        stack.set_data_component(InstrumentImpl {
            instrument: Cow::Borrowed("minecraft:ponder_goat_horn"),
        });
        assert_eq!(
            GoatHornItem.get_use_duration(&stack),
            i32::try_from(Instrument::PonderGoatHorn.use_duration_ticks()).unwrap_or(i32::MAX)
        );
    }
}
