use anyhow::{anyhow, Result};
use sha1::{Digest, Sha1};
use std::io::{Read, Write};
use std::net::TcpStream;

#[derive(Debug, Clone)]
pub struct ReplicationConfig {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub password: String,
    pub server_id: u32,
    pub binlog_file: Option<String>,
    pub binlog_pos: Option<u32>,
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
}

impl BinlogConnector for MysqlConnector {
    fn connect(&mut self) -> Result<()> {
        let addr = format!("{}:{}", self.config.host, self.config.port);
        let mut stream = TcpStream::connect(addr)?;

        let mut handshake_buf = [0u8; 256];
        let len = stream.read(&mut handshake_buf)?;
        let data = &handshake_buf[..len];

        // Parse salt from handshake packet (assumes mysql_native_password)
        let scramble = {
            let mut salt = Vec::new();
            salt.extend_from_slice(&data[15..23]);
            salt.extend_from_slice(&data[35..43]);
            salt
        };

        let password = self.config.password.as_bytes();
        let hashed = {
            let hash1 = Sha1::digest(password);
            let hash2 = Sha1::digest(&hash1);
            let mut input = scramble.clone();
            input.extend_from_slice(&hash2);
            let hash3 = Sha1::digest(&input);
            hash1.iter().zip(hash3).map(|(a, b)| a ^ b).collect::<Vec<u8>>()
        };

        // Compose basic handshake response packet
        let mut response = Vec::new();
        response.extend_from_slice(&[0x85, 0xa6, 0x3f, 0x00]); // client flags
        response.extend_from_slice(&[0x00, 0x00, 0x00, 0x01]); // max packet size + charset
        response.extend_from_slice(&[0u8; 23]); // filler
        response.extend_from_slice(self.config.user.as_bytes());
        response.push(0x00);
        response.push(hashed.len() as u8);
        response.extend_from_slice(&hashed);
        response.push(0x00);

        let mut packet = Vec::new();
        let size = response.len() as u32;
        packet.extend_from_slice(&(size as u8).to_le_bytes());
        packet.push(0x01); // sequence id
        packet.extend_from_slice(&response);

        stream.write_all(&packet)?;

        let mut ok_buf = [0u8; 8];
        stream.read(&mut ok_buf)?;
        if ok_buf[0] != 0x00 {
            return Err(anyhow!("MySQL auth failed: {:02X?}", ok_buf));
        }

        self.stream = Some(stream);
        Ok(())
    }

    fn start_replication(&mut self) -> Result<()> {
        Ok(())
    }

    fn read_packet(&mut self) -> Result<Vec<u8>> {
        Ok(vec![])
    }
}
