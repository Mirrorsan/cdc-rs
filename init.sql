-- Create replication user with proper privileges
CREATE USER IF NOT EXISTS 'repl'@'%' IDENTIFIED BY 'replpass';
GRANT REPLICATION SLAVE, REPLICATION CLIENT ON *.* TO 'repl'@'%';
GRANT SELECT ON *.* TO 'repl'@'%';
FLUSH PRIVILEGES;

-- Create test database and table
USE testdb;

CREATE TABLE IF NOT EXISTS users (
    id INT AUTO_INCREMENT PRIMARY KEY,
    name VARCHAR(100) NOT NULL,
    email VARCHAR(100) UNIQUE,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Insert some test data
INSERT INTO users (name, email) VALUES 
    ('Alice Johnson', 'alice@example.com'),
    ('Bob Smith', 'bob@example.com'),
    ('Carol Wilson', 'carol@example.com');

-- Show replication status
SHOW MASTER STATUS;

-- Show users and their privileges
SELECT User, Host, plugin FROM mysql.user WHERE User = 'repl';
SHOW GRANTS FOR 'repl'@'%';