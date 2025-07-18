/// Enum representing a high-level binlog event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BinlogEvent {
    Insert { table: String, row: Vec<(String, String)> },
    Update { table: String, old_row: Vec<(String, String)>, new_row: Vec<(String, String)> },
    Delete { table: String, row: Vec<(String, String)> },
    Other(String),
}
