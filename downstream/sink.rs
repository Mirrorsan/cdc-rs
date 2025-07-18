use crate::binlog::events::BinlogEvent;
use anyhow::Result;

/// Result type for sinks.
pub type SinkResult = Result<()>;

/// Trait representing a consumer of CDC events.
pub trait EventSink {
    /// Handle a single binlog event.
    fn handle_event(&mut self, event: BinlogEvent) -> SinkResult;
}
