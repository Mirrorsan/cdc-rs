use cdc_rs::{Replicator, BinlogEvent, EventSink, SinkResult};

#[cfg(test)]
mod tests {
    use super::*;

    struct TestSink {
        pub events: Vec<BinlogEvent>,
    }

    impl TestSink {
        fn new() -> Self {
            Self { events: Vec::new() }
        }
    }

    impl EventSink for TestSink {
        fn handle_event(&mut self, event: BinlogEvent) -> SinkResult {
            self.events.push(event);
            Ok(())
        }
    }

    #[test]
    fn test_replicator_run_with_test_sink() {
        let mut sink = TestSink::new();
        let result = Replicator::run(&mut sink);

        assert!(result.is_ok());
        assert_eq!(sink.events.len(), 4);
        assert!(matches!(sink.events[0], BinlogEvent::Insert { .. }));
        assert!(matches!(sink.events[1], BinlogEvent::Update { .. }));
        assert!(matches!(sink.events[2], BinlogEvent::Delete { .. }));
        assert!(matches!(sink.events[3], BinlogEvent::Other(_)));
    }
}