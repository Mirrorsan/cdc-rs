#[cfg(test)]
mod tests {
    use crate::binlog::events::BinlogEvent;
    use crate::binlog::parser::{BinlogParser, MockBinlogParser};

    #[test]
    fn test_insert_event() {
        let mut parser = MockBinlogParser;
        let input = b"insert into table";
        let event = parser.parse(input).unwrap();

        match event {
            BinlogEvent::Insert { table, row } => {
                assert_eq!(table, "mock_table");
                assert_eq!(row[0], ("id".into(), "1".into()));
            }
            _ => panic!("Expected Insert"),
        }
    }

    #[test]
    fn test_unknown_event() {
        let mut parser = MockBinlogParser;
        let input = b"ping";
        let event = parser.parse(input).unwrap();

        match event {
            BinlogEvent::Other(ref kind) => assert_eq!(kind, "ping"),
            _ => panic!("Expected Other event"),
        }
    }
}
