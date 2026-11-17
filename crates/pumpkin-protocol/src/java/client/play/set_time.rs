use pumpkin_data::packet::clientbound::play::SET_TIME;
use pumpkin_macros::java_packet;
use pumpkin_util::version::JavaMinecraftVersion;

use crate::{
    ClientPacket,
    codec::{var_int::VarInt, var_long::VarLong},
    ser::{NetworkWriteExt, WritingError},
};

#[java_packet(SET_TIME)]
pub struct CUpdateTime {
    pub game_time: i64,
    /// (`clock_registry_id`, `total_ticks`, `partial_tick`, rate)
    pub clock_updates: Vec<(i32, i64, f32, f32)>,
    /// Day time and `doDaylightCycle` for clients before 26.1, which have no
    /// clock map and read both from dedicated fields. `None` falls back to the
    /// first clock update, then to day 0.
    pub legacy_day_time: Option<(i64, bool)>,
}

impl CUpdateTime {
    #[must_use]
    pub fn new(game_time: i64, day_time: i64, increasing: bool) -> Self {
        let overworld_id = 0;
        let partial_tick = 0.0f32;
        let rate = if increasing { 1.0f32 } else { 0.0f32 }; // Normal speed
        Self {
            game_time,
            clock_updates: vec![(overworld_id, day_time, partial_tick, rate)],
            legacy_day_time: None,
        }
    }

    #[must_use]
    pub fn new_clock(
        game_time: i64,
        clock_id: i32,
        total_ticks: i64,
        partial_tick: f32,
        rate: f32,
    ) -> Self {
        Self {
            game_time,
            clock_updates: vec![(clock_id, total_ticks, partial_tick, rate)],
            legacy_day_time: None,
        }
    }
}

impl ClientPacket for CUpdateTime {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        version: &JavaMinecraftVersion,
    ) -> Result<(), WritingError> {
        write.write_i64_be(self.game_time)?;

        if version >= &JavaMinecraftVersion::V_26_1 {
            write.write_var_int(&VarInt(self.clock_updates.len() as i32))?;
            for &(clock_id, total_ticks, partial_tick, rate) in &self.clock_updates {
                write.write_var_int(&VarInt(clock_id))?;
                write.write_var_long(&VarLong(total_ticks))?;
                write.write_f32_be(partial_tick)?;
                write.write_f32_be(rate)?;
            }
        } else {
            let (day_time, rate) = self.legacy_day_time.map_or_else(
                || {
                    self.clock_updates
                        .first()
                        .map_or((0, 1.0), |&(_, total_ticks, _, rate)| (total_ticks, rate))
                },
                |(day_time, increasing)| (day_time, if increasing { 1.0 } else { 0.0 }),
            );

            let is_paused = rate == 0.0;
            let day_time = if version < &JavaMinecraftVersion::V_1_21_2 {
                if is_paused {
                    match day_time.cmp(&0) {
                        std::cmp::Ordering::Greater => -day_time,
                        std::cmp::Ordering::Equal => -1,
                        std::cmp::Ordering::Less => day_time,
                    }
                } else {
                    day_time.abs()
                }
            } else {
                day_time
            };

            write.write_i64_be(day_time)?;

            if version >= &JavaMinecraftVersion::V_1_21_2 {
                let is_increasing = rate > 0.0;
                write.write_bool(is_increasing)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn legacy_sync_writes_day_time_and_frozen_flag() {
        let packet = CUpdateTime {
            game_time: 1_000,
            clock_updates: Vec::new(),
            legacy_day_time: Some((13_000, false)),
        };
        let mut buf = Vec::new();
        packet
            .write_packet_data(&mut buf, &JavaMinecraftVersion::V_1_21_11)
            .unwrap();
        // Vanilla ClientboundSetTimePacket(gameTime, dayTime, doDaylightCycle).
        assert_eq!(&buf[0..8], &1_000i64.to_be_bytes());
        assert_eq!(&buf[8..16], &13_000i64.to_be_bytes());
        assert_eq!(buf[16], 0);
        assert_eq!(buf.len(), 17);
    }

    #[test]
    fn clock_sync_omits_legacy_fields_for_current_clients() {
        let packet = CUpdateTime {
            game_time: 1_000,
            clock_updates: Vec::new(),
            legacy_day_time: Some((13_000, false)),
        };
        let mut buf = Vec::new();
        packet
            .write_packet_data(&mut buf, &JavaMinecraftVersion::V_26_3)
            .unwrap();
        assert_eq!(&buf[0..8], &1_000i64.to_be_bytes());
        assert_eq!(buf.len(), 9);
    }
}
