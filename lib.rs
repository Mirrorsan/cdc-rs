pub mod binlog;
pub mod replication;
pub mod downstream;

// Optional re-exports for end users
pub use replication::replicator::Replicator;
pub use downstream::sink::{EventSink, SinkResult};
pub use binlog::events::BinlogEvent;
pub use binlog::parser::{BinlogParser, MockBinlogParser};