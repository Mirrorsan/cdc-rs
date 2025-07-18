use crate::binlog::events::BinlogEvent;
use crate::downstream::sink::EventSink;
use anyhow::Result;

/// Top-level interface to run replication from MySQL binlogs.
pub struct Replicator;

impl Replicator {
    /// Synchronous replication runner.
    pub fn run<S: EventSink>(sink: &mut S) -> Result<()> {
        // Placeholder: real implementation will read and parse binlog stream
        Ok(())
    }

    /// Asynchronous replication runner (enabled with `async` feature).
    #[cfg(feature = "async")]
    pub async fn run_async<S: EventSink + Send>(sink: &mut S) -> Result<()> {
        // Placeholder: real implementation will read and parse binlog stream
        Ok(())
    }
}