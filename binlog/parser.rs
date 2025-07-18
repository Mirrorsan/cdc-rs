use crate::binlog::events::BinlogEvent;

/// Trait for parsing raw binlog bytes into structured events.
pub trait BinlogParser {
    /// Attempts to parse a binlog event from the given byte slice.
    fn parse(&mut self, input: &[u8]) -> Option<BinlogEvent>;
}

/// A simple stub parser that interprets lines of text as mock events.
/// Used for early testing and development.
pub struct MockBinlogParser;

impl BinlogParser for MockBinlogParser {
    fn parse(&mut self, input: &[u8]) -> Option<BinlogEvent> {
        let line = std::str::from_utf8(input).ok()?.trim();

        if line.starts_with("insert") {
            Some(BinlogEvent::Insert {
                table: "mock_table".into(),
                row: vec![("id".into(), "1".into())],
            })
        } else if line.starts_with("update") {
            Some(BinlogEvent::Update {
                table: "mock_table".into(),
                old_row: vec![("id".into(), "1".into())],
                new_row: vec![("id".into(), "2".into())],
            })
        } else if line.starts_with("delete") {
            Some(BinlogEvent::Delete {
                table: "mock_table".into(),
                row: vec![("id".into(), "1".into())],
            })
        } else {
            Some(BinlogEvent::Other(line.to_string()))
        }
    }
}