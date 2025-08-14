// replication/protocol.rs
use sha1::{Digest, Sha1};

/// Capability flags (Protocol::41)
/// Source: MySQL 5.7/8.0, MariaDB 10.x headers.
const CLIENT_LONG_PASSWORD: u32 = 0x0000_0001;
#[allow(dead_code)]
const CLIENT_FOUND_ROWS: u32 = 0x0000_0002;
const CLIENT_LONG_FLAG: u32 = 0x0000_0004;
const CLIENT_CONNECT_WITH_DB: u32 = 0x0000_0008;
const CLIENT_PROTOCOL_41: u32 = 0x0000_0200;
const CLIENT_TRANSACTIONS: u32 = 0x0000_2000;
const CLIENT_SECURE_CONNECTION: u32 = 0x0000_8000;
const CLIENT_MULTI_RESULTS: u32 = 0x0002_0000;
const CLIENT_PLUGIN_AUTH: u32 = 0x0008_0000;
#[allow(dead_code)]
const CLIENT_CONNECT_ATTRS: u32 = 0x0010_0000;
const CLIENT_PLUGIN_AUTH_LENENC_CLIENT_DATA: u32 = 0x0020_0000;
const CLIENT_DEPRECATE_EOF: u32 = 0x0100_0000;

/// Scrambles a password using the MySQL `mysql_native_password` auth scheme.
pub fn scramble_password(salt: &[u8], password: &str) -> Vec<u8> {
    if password.is_empty() {
        return Vec::new();
    }

    let pass = password.as_bytes();
    let hash1 = Sha1::digest(pass);
    let hash2 = Sha1::digest(&hash1);

    let mut input = Vec::with_capacity(salt.len() + hash2.len());
    input.extend_from_slice(salt);
    input.extend_from_slice(&hash2);

    let hash3 = Sha1::digest(&input);

    hash1.iter().zip(hash3.iter()).map(|(a, b)| a ^ b).collect()
}

/// Write a length-encoded integer as used by MySQL protocol.
fn write_lenenc_int(buf: &mut Vec<u8>, v: u64) {
    if v < 251 {
        buf.push(v as u8);
    } else if v < (1 << 16) {
        buf.push(0xFC);
        buf.extend_from_slice(&(v as u16).to_le_bytes());
    } else if v < (1 << 24) {
        buf.push(0xFD);
        let bytes = (v as u32).to_le_bytes();
        buf.extend_from_slice(&bytes[..3]);
    } else {
        buf.push(0xFE);
        buf.extend_from_slice(&v.to_le_bytes());
    }
}

/// Builds a valid MySQL login request packet (Handshake Response 41).
pub fn build_login_packet(username: &str, password: &str, salt: &[u8], database: Option<&str>) -> Vec<u8> {
    let scrambled = scramble_password(salt, password);
    println!("Scrambled password: {:02X?}", scrambled);

    // Conservative, widely-supported capability set.
    let mut client_flags: u32 =
          CLIENT_LONG_PASSWORD
        | CLIENT_LONG_FLAG
        | CLIENT_PROTOCOL_41
        | CLIENT_SECURE_CONNECTION
        | CLIENT_PLUGIN_AUTH
        | CLIENT_PLUGIN_AUTH_LENENC_CLIENT_DATA
        | CLIENT_TRANSACTIONS
        | CLIENT_MULTI_RESULTS
        | CLIENT_DEPRECATE_EOF;
        // NOTE: avoid CLIENT_CONNECT_ATTRS unless we actually send attrs.

    if database.is_some() {
        client_flags |= CLIENT_CONNECT_WITH_DB;
    }

    let max_packet_size: u32 = 16 * 1024 * 1024;
    let charset: u8 = 0x21; // utf8_general_ci (33)
    let plugin_name = b"mysql_native_password";

    let mut payload = Vec::with_capacity(256);

    // 4 bytes: client_flags
    payload.extend_from_slice(&client_flags.to_le_bytes());

    // 4 bytes: max_packet_size
    payload.extend_from_slice(&max_packet_size.to_le_bytes());

    // 1 byte: character_set
    payload.push(charset);

    // 23 bytes: filler (all zeros)
    payload.extend_from_slice(&[0u8; 23]);

    // username (null-terminated)
    payload.extend_from_slice(username.as_bytes());
    payload.push(0x00);

    // auth response
    if (client_flags & CLIENT_PLUGIN_AUTH_LENENC_CLIENT_DATA) != 0 {
        write_lenenc_int(&mut payload, scrambled.len() as u64);
        payload.extend_from_slice(&scrambled);
    } else if (client_flags & CLIENT_SECURE_CONNECTION) != 0 {
        payload.push(scrambled.len() as u8);
        payload.extend_from_slice(&scrambled);
    } else {
        // old, nul-terminated style
        payload.extend_from_slice(&scrambled);
        payload.push(0x00);
    }

    // optional database
    if let Some(db) = database {
        payload.extend_from_slice(db.as_bytes());
        payload.push(0x00);
    }

    // plugin name (required when CLIENT_PLUGIN_AUTH set)
    if (client_flags & CLIENT_PLUGIN_AUTH) != 0 {
        payload.extend_from_slice(plugin_name);
        payload.push(0x00);
    }

    // final packet with header
    let payload_len = payload.len() as u32;

    let mut packet = Vec::with_capacity(payload_len as usize + 4);
    // 3-byte little-endian length
    let len_bytes = payload_len.to_le_bytes();
    packet.extend_from_slice(&len_bytes[..3]);
    // sequence id
    packet.push(0x01);
    // payload
    packet.extend_from_slice(&payload);

    println!("Built packet - Total length: {}, Payload length: {}", packet.len(), payload_len);

    packet
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_login_packet_contains_username_and_auth() {
        // fixed salt (20 bytes)
        let salt = b"12345678abcdefghij"; // 20 bytes
        let pkt = build_login_packet("repl", "replpass", salt, None);

        // header is 4 bytes; search for 'repl\\0' in payload
        assert!(pkt.windows(5).any(|w| w == b"repl\\0"), "username not present");
        // plugin name present
        assert!(pkt.windows(b"mysql_native_password".len()).any(|w| w == b"mysql_native_password"), "plugin name missing");
        // payload len matches header (3 bytes little-endian)
        let payload_len = pkt[0] as u32 | ((pkt[1] as u32) << 8) | ((pkt[2] as u32) << 16);
        assert_eq!(payload_len as usize, pkt.len() - 4);
        // sequence id is 1
        assert_eq!(pkt[3], 0x01);
    }
}
