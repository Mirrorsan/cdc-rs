// tests/connect.rs
use cdc_rs::replication::connector::{MysqlConnector, ReplicationConfig, BinlogConnector};
use std::time::Duration;
use std::thread;

#[test]
fn test_connect_to_mysql() {
    // Wait a bit to ensure MySQL is fully ready
    thread::sleep(Duration::from_secs(2));
    
    let config = ReplicationConfig {
        host: "127.0.0.1".into(),
        port: 3306,
        user: "repl".into(),
        password: "replpass".into(),
        server_id: 1234,
        binlog_file: None,
        binlog_pos: None,
        database: Some("testdb".into()),
    };
    
    let mut connector = MysqlConnector::new(config);
    let result = connector.connect();
    
    match result {
        Ok(()) => {
            println!("✅ Successfully connected to MySQL!");
        }
        Err(e) => {
            println!("❌ Failed to connect to MySQL: {}", e);
            panic!("Connection failed: {}", e);
        }
    }
}

#[test]
fn test_connect_without_database() {
    thread::sleep(Duration::from_secs(1));
    
    let config = ReplicationConfig {
        host: "127.0.0.1".into(),
        port: 3306,
        user: "repl".into(),
        password: "replpass".into(),
        server_id: 1234,
        binlog_file: None,
        binlog_pos: None,
        database: None, // No database specified
    };
    
    let mut connector = MysqlConnector::new(config);
    let result = connector.connect();
    
    match result {
        Ok(()) => {
            println!("✅ Successfully connected to MySQL without database!");
        }
        Err(e) => {
            println!("❌ Failed to connect to MySQL: {}", e);
            panic!("Connection failed: {}", e);
        }
    }
}

#[test]
fn test_wrong_password() {
    thread::sleep(Duration::from_secs(1));
    
    let config = ReplicationConfig {
        host: "127.0.0.1".into(),
        port: 3306,
        user: "repl".into(),
        password: "wrongpassword".into(), // Intentionally wrong password
        server_id: 1234,
        binlog_file: None,
        binlog_pos: None,
        database: Some("testdb".into()),
    };
    
    let mut connector = MysqlConnector::new(config);
    let result = connector.connect();
    
    // This should fail
    assert!(result.is_err(), "Expected authentication to fail with wrong password");
    println!("✅ Correctly rejected wrong password: {}", result.unwrap_err());
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use std::process::Command;
    
    #[test]
    #[ignore] // Run with: cargo test test_with_docker_setup -- --ignored
    fn test_with_docker_setup() {
        // Start Docker containers
        println!("Starting Docker containers...");
        let output = Command::new("docker-compose")
            .args(&["up", "-d"])
            .output()
            .expect("Failed to start docker-compose");
            
        if !output.status.success() {
            panic!("Failed to start Docker containers: {}", 
                   String::from_utf8_lossy(&output.stderr));
        }
        
        // Wait for MySQL to be ready
        thread::sleep(Duration::from_secs(10));
        
        // Run the actual test
        test_connect_to_mysql();
        
        // Cleanup
        println!("Stopping Docker containers...");
        let _ = Command::new("docker-compose")
            .args(&["down"])
            .output();
    }
}