use std::sync::Arc;

use pumpkin_util::math::vector2::Vector2;
use pumpkin_world::{chunk_system::ChunkLoading, level::Level};

/// Keeps a Creaking's heart chunk loaded while the mob is alive, mirroring the
/// allay's note-block chunk ticket. Dropping it releases the load request.
pub(super) struct HeartChunkTicket {
    pub(super) level: Arc<Level>,
    pub(super) chunk: Vector2<i32>,
}

impl HeartChunkTicket {
    pub(super) fn new(level: Arc<Level>, chunk: Vector2<i32>) -> Self {
        let mut loading = level
            .chunk_loading
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        loading.add_ticket(chunk, ChunkLoading::FULL_CHUNK_LEVEL);
        loading.send_change();
        drop(loading);
        Self { level, chunk }
    }
}

impl Drop for HeartChunkTicket {
    fn drop(&mut self) {
        let mut loading = self
            .level
            .chunk_loading
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        loading.remove_ticket(self.chunk, ChunkLoading::FULL_CHUNK_LEVEL);
        loading.send_change();
    }
}
