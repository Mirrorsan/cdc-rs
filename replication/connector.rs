// replication/connector.rs
use anyhow::{anyhow, Result};
use std::io::{Read, Write};
use std::net::TcpStream;
use crate::replication::protocol::build_login_packet;

#[derive(Debug, Clone)]
pub struct ReplicationConfig {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub password: String,
    pub server_id: u32,
    pub binlog_file: Option<String>,
    pub binlog_pos: Option<u32>,
    pub database: Option<String>,
}

pub trait BinlogConnector {
    fn connect(&mut self) -> Result<()>;
    fn start_replication(&mut self) -> Result<()>;
    fn read_packet(&mut self) -> Result<Vec<u8>>;
}

pub struct MysqlConnector {
    config: ReplicationConfig,
    stream: Option<TcpStream>,
}

impl MysqlConnector {
    pub fn new(config: ReplicationConfig) -> Self {
        Self { config, stream: None }
    }

    /// Read a complete MySQL packet (4-byte header + payload)
    fn read_mysql_packet(stream: &mut TcpStream) -> Result<Vec<u8>> {
        let mut header = [0u8; 4];
        stream.read_exact(&mut header)?;
        
        let payload_len = u32::from_le_bytes([header[0], header[1], header[2], 0]) as usize;
        let _sequence_id = header[3];
        
        let mut payload = vec![0u8; payload_len];
        stream.read_exact(&mut payload)?;
        
        Ok(payload)
    }
}

impl BinlogConnector for MysqlConnector {
    fn connect(&mut self) -> Result<()> {
        let addr = format!("{}:{}", self.config.host, self.config.port);
        let mut stream = TcpStream::connect(addr)?;

        // Read handshake packet
        let handshake_payload = Self::read_mysql_packet(&mut stream)?;
        println!("Handshake payload length: {}", handshake_payload.len());

        // Parse handshake packet more robustly
        let scramble = self.parse_handshake(&handshake_payload)?;
        println!("Scramble (20 bytes): {:02X?}", scramble);

        // Build and send login packet
        let packet = build_login_packet(
            &self.config.user,
            &self.config.password,
            &scramble,
            self.config.database.as_deref(),
        );
        
        println!("Sending login packet ({} bytes)", packet.len());
        stream.write_all(&packet)?;
        stream.flush()?;

        // Read authentication response
        let response = Self::read_mysql_packet(&mut stream)?;
        println!("Auth response: {:02X?}", response);

        // Check if authentication succeeded
        match response.first() {
            Some(0x00) => {
                println!("Authentication successful!");
                self.stream = Some(stream);
                Ok(())
            }
            Some(0xFF) => {
                // Error packet - extract error message
                if response.len() >= 3 {
                    let error_code = u16::from_le_bytes([response[1], response[2]]);
                    let error_msg = if response.len() > 9 {
                        String::from_utf8_lossy(&response[9..])
                    } else {
                        "Unknown error".into()
                    };
                    Err(anyhow!("MySQL auth failed: Error {} - {}", error_code, error_msg))
                } else {
                    Err(anyhow!("MySQL auth failed: Malformed error packet"))
                }
            }
            Some(0xFE) => {
                Err(anyhow!("MySQL auth failed: Auth method switch required"))
            }
            _ => {
                Err(anyhow!("MySQL auth failed: Unexpected response: {:02X?}", response))
            }
        }
    }

    fn start_replication(&mut self) -> Result<()> {
        Ok(())
    }

    fn read_packet(&mut self) -> Result<Vec<u8>> {
        Ok(vec![])
    }
}

impl MysqlConnector {
    fn parse_handshake(&self, payload: &[u8]) -> Result<Vec<u8>> {
        if payload.len() < 32 {
            return Err(anyhow!("Handshake packet too short"));
        }

        let mut pos = 0;

        // Protocol version (1 byte)
        let protocol_version = payload[pos];
        pos += 1;
        println!("Protocol version: {}", protocol_version);

        // Server version (null-terminated string)
        let version_start = pos;
        while pos < payload.len() && payload[pos] != 0 {
            pos += 1;
        }
        let server_version = String::from_utf8_lossy(&payload[version_start..pos]);
        pos += 1; // skip null terminator
        println!("Server version: {}", server_version);

        // Connection ID (4 bytes)
        if pos + 4 > payload.len() {
            return Err(anyhow!("Invalid handshake: connection ID"));
        }
        pos += 4;

        // Auth plugin data part 1 (8 bytes)
        if pos + 8 > payload.len() {
            return Err(anyhow!("Invalid handshake: auth data part 1"));
        }
        let auth_data_part1 = &payload[pos..pos + 8];
        pos += 8;

        // Filler (1 byte)
        pos += 1;

        // Server capabilities lower 16 bits (2 bytes)
        if pos + 2 > payload.len() {
            return Err(anyhow!("Invalid handshake: server capabilities"));
        }
        pos += 2;

        // Server character set (1 byte)
        pos += 1;

        // Server status flags (2 bytes)
        pos += 2;

        // Server capabilities upper 16 bits (2 bytes)
        pos += 2;

        // Length of auth plugin data (1 byte) or 0
        let auth_data_len = if pos < payload.len() {
            payload[pos]
        } else {
            0
        };
        pos += 1;

        // Reserved (10 bytes)
        pos += 10;

        // Auth plugin data part 2
        let auth_data_part2_len = if auth_data_len > 8 {
            (auth_data_len - 8) as usize
        } else {
            12 // Default for mysql_native_password
        };

        if pos + auth_data_part2_len > payload.len() {
            return Err(anyhow!("Invalid handshake: auth data part 2"));
        }

        let auth_data_part2 = &payload[pos..pos + auth_data_part2_len];

        // Combine both parts of the scramble (remove trailing null if present)
        let mut scramble = Vec::with_capacity(20);
        scramble.extend_from_slice(auth_data_part1);
        scramble.extend_from_slice(auth_data_part2);
        
        // Remove trailing null byte if present
        if scramble.len() > 0 && scramble[scramble.len() - 1] == 0 {
            scramble.pop();
        }

        // Ensure we have exactly 20 bytes for mysql_native_password
        if scramble.len() != 20 {
            return Err(anyhow!("Invalid scramble length: {} (expected 20)", scramble.len()));
        }

        Ok(scramble)
    }
}