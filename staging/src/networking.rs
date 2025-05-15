//! Networking Examples
//!
//! This module demonstrates common networking patterns with tokio:
//! - Asynchronous TCP connections
//! - HTTP clients and servers
//! - WebSockets
//! - UDP communication
//! - Connection pooling
//! - Error handling in network operations

use std::io;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use futures::StreamExt;
use std::sync::Arc;
use std::time::Duration;
use std::collections::HashMap;

/// Problem: Create a simple TCP echo server
///
/// Implement an asynchronous TCP server that echoes back client messages
pub async fn run_tcp_echo_server(address: &str) -> io::Result<()> {
    // Bind the TCP listener to the address
    let listener = TcpListener::bind(address).await?;
    println!("Listening on: {}", address);

    loop {
        // Accept a connection
        let (mut socket, addr) = listener.accept().await?;
        println!("Accepted connection from: {}", addr);

        // Spawn a task to handle the connection
        tokio::spawn(async move {
            let mut buf = vec![0; 1024];

            // Read and echo loop
            loop {
                let n = match socket.read(&mut buf).await {
                    Ok(0) => break, // Connection closed
                    Ok(n) => n,
                    Err(e) => {
                        eprintln!("Failed to read from socket: {}", e);
                        break;
                    }
                };

                // Echo the data back
                if let Err(e) = socket.write_all(&buf[0..n]).await {
                    eprintln!("Failed to write to socket: {}", e);
                    break;
                }
            }

            println!("Connection closed: {}", addr);
        });
    }
}

/// Problem: Implement a simple TCP client
///
/// Create an asynchronous client that connects to a TCP server
pub async fn tcp_client_send(address: &str, message: &str) -> io::Result<String> {
    // Connect to server
    let mut stream = TcpStream::connect(address).await?;
    println!("Connected to: {}", address);

    // Send data
    stream.write_all(message.as_bytes()).await?;
    println!("Sent: {}", message);

    // Read the response
    let mut buffer = vec![0; 1024];
    let n = stream.read(&mut buffer).await?;
    let response = String::from_utf8_lossy(&buffer[..n]).to_string();

    Ok(response)
}

/// Problem: Create a connection pool
///
/// Implement a simple connection pool for TCP connections
#[derive(Debug)]
pub struct ConnectionPool {
    connections: Arc<tokio::sync::Mutex<HashMap<String, Vec<TcpStream>>>>,
    max_connections: usize,
    timeout: Duration,
}

impl ConnectionPool {
    pub fn new(max_connections: usize, timeout_secs: u64) -> Self {
        ConnectionPool {
            connections: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
            max_connections,
            timeout: Duration::from_secs(timeout_secs),
        }
    }

    pub async fn get_connection(&self, address: &str) -> io::Result<TcpStream> {
        let addr_key = address.to_string();
        let mut connections = self.connections.lock().await;

        // Check if we have a connection available
        if let Some(conn_list) = connections.get_mut(&addr_key) {
            if let Some(stream) = conn_list.pop() {
                return Ok(stream);
            }
        }

        // No connection available, create a new one
        let stream = tokio::time::timeout(
            self.timeout,
            TcpStream::connect(address),
        ).await??;

        Ok(stream)
    }

    pub async fn release_connection(&self, address: &str, stream: TcpStream) {
        let addr_key = address.to_string();
        let mut connections = self.connections.lock().await;

        let conn_list = connections.entry(addr_key).or_insert_with(Vec::new);
        if conn_list.len() < self.max_connections {
            conn_list.push(stream);
        }
        // If at capacity, the stream is dropped and closed
    }

    pub async fn execute<F, R>(&self, address: &str, operation: F) -> io::Result<R>
    where
        F: FnOnce(TcpStream) -> futures::future::BoxFuture<'static, io::Result<(R, TcpStream)>>,
    {
        let stream = self.get_connection(address).await?;
        let addr = address.to_string();
        let pool = self.clone();

        // Execute the operation and return the connection to the pool
        let (result, stream) = operation(stream).await?;
        pool.release_connection(&addr, stream).await;

        Ok(result)
    }
}

impl Clone for ConnectionPool {
    fn clone(&self) -> Self {
        ConnectionPool {
            connections: Arc::clone(&self.connections),
            max_connections: self.max_connections,
            timeout: self.timeout,
        }
    }
}

/// Problem: Implement a simple HTTP client
///
/// Create a function that makes an HTTP request and returns the response
pub async fn http_get_request(url: &str) -> Result<String, String> {
    // This is a simplified version that would normally use a proper HTTP client
    // For this example, we're manually crafting a basic HTTP request

    // Parse the URL to get the host and path
    let url = url::Url::parse(url).map_err(|e| format!("Invalid URL: {}", e))?;
    let host = url.host_str().ok_or("Missing host")?;
    let port = url.port().unwrap_or(80);
    let path = url.path();

    // Format the HTTP request
    let request = format!(
        "GET {} HTTP/1.1\r\n\
         Host: {}\r\n\
         Connection: close\r\n\
         \r\n",
        path, host
    );

    // Connect to the server
    let address = format!("{}:{}", host, port);
    let mut stream = TcpStream::connect(&address)
        .await
        .map_err(|e| format!("Connection error: {}", e))?;

    // Send the request
    stream
        .write_all(request.as_bytes())
        .await
        .map_err(|e| format!("Failed to send request: {}", e))?;

    // Read the response
    let mut buffer = Vec::new();
    stream
        .read_to_end(&mut buffer)
        .await
        .map_err(|e| format!("Failed to read response: {}", e))?;

    // Convert the response to a string
    let response = String::from_utf8_lossy(&buffer).to_string();

    Ok(response)
}

/// Problem: Create a chat server using WebSockets
///
/// This is a simplified representation of what a WebSocket chat server might look like
/// using tokio. In a real implementation, you would use a proper WebSocket library.
pub struct ChatServer {
    clients: Arc<tokio::sync::Mutex<HashMap<String, mpsc::UnboundedSender<String>>>>,
}

use tokio::sync::mpsc;

impl ChatServer {
    pub fn new() -> Self {
        ChatServer {
            clients: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
        }
    }

    pub async fn add_client(&self, client_id: &str) -> mpsc::UnboundedReceiver<String> {
        let (tx, rx) = mpsc::unbounded_channel();
        let mut clients = self.clients.lock().await;
        clients.insert(client_id.to_string(), tx);
        rx
    }

    pub async fn remove_client(&self, client_id: &str) {
        let mut clients = self.clients.lock().await;
        clients.remove(client_id);
    }

    pub async fn broadcast_message(&self, from_client: &str, message: &str) {
        let message = format!("{}: {}", from_client, message);
        let clients = self.clients.lock().await;

        for (client_id, tx) in clients.iter() {
            if client_id != from_client {
                let _ = tx.send(message.clone());
            }
        }
    }
}

/// Problem: Implement UDP communication
///
/// Create functions for sending and receiving UDP datagrams
pub async fn udp_send(address: &str, message: &str) -> io::Result<()> {
    use tokio::net::UdpSocket;

    // Bind to any available local address
    let socket = UdpSocket::bind("0.0.0.0:0").await?;

    // Send the datagram
    socket.send_to(message.as_bytes(), address).await?;

    Ok(())
}

pub async fn udp_receive(address: &str) -> io::Result<(String, String)> {
    use tokio::net::UdpSocket;

    // Bind to the specified address
    let socket = UdpSocket::bind(address).await?;

    // Receive a datagram
    let mut buf = vec![0; 1024];
    let (size, peer_addr) = socket.recv_from(&mut buf).await?;

    // Convert to string
    let message = String::from_utf8_lossy(&buf[..size]).to_string();

    Ok((message, peer_addr.to_string()))
}

/// Problem: Implement a rate limiter for network requests
///
/// Create a token bucket rate limiter for network operations
pub struct RateLimiter {
    tokens: Arc<tokio::sync::Mutex<u32>>,
    max_tokens: u32,
    refill_interval: Duration,
    tokens_per_refill: u32,
}

impl RateLimiter {
    pub fn new(max_tokens: u32, refill_interval_ms: u64, tokens_per_refill: u32) -> Self {
        let limiter = RateLimiter {
            tokens: Arc::new(tokio::sync::Mutex::new(max_tokens)),
            max_tokens,
            refill_interval: Duration::from_millis(refill_interval_ms),
            tokens_per_refill,
        };

        // Start the token refill task
        let tokens_clone = Arc::clone(&limiter.tokens);
        let max_tokens = limiter.max_tokens;
        let refill_interval = limiter.refill_interval;
        let tokens_per_refill = limiter.tokens_per_refill;

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(refill_interval);

            loop {
                interval.tick().await;
                let mut tokens = tokens_clone.lock().await;
                *tokens = (*tokens + tokens_per_refill).min(max_tokens);
            }
        });

        limiter
    }

    pub async fn acquire(&self, tokens_needed: u32) -> bool {
        let mut available_tokens = self.tokens.lock().await;

        if *available_tokens >= tokens_needed {
            *available_tokens -= tokens_needed;
            true
        } else {
            false
        }
    }

    pub async fn execute<F, R>(&self, tokens_needed: u32, operation: F) -> io::Result<R>
    where
        F: FnOnce() -> futures::future::BoxFuture<'static, io::Result<R>>,
    {
        // Try to acquire tokens
        if !self.acquire(tokens_needed).await {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "Rate limit exceeded",
            ));
        }

        // Execute the operation
        operation().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // We can't easily test actual network operations in unit tests,
    // but we can demonstrate how these functions would be used

    #[test]
    fn demonstrate_usage() {
        // This test doesn't actually run the async functions,
        // it just demonstrates how they would be used

        println!("TCP echo server usage:");
        println!("tokio::spawn(run_tcp_echo_server(\"127.0.0.1:8080\"));");

        println!("\nTCP client usage:");
        println!("let response = tcp_client_send(\"127.0.0.1:8080\", \"Hello, server!\").await?;");

        println!("\nConnection pool usage:");
        println!("let pool = ConnectionPool::new(10, 5);");
        println!("let connection = pool.get_connection(\"127.0.0.1:8080\").await?;");
        println!("// Use connection");
        println!("pool.release_connection(\"127.0.0.1:8080\", connection).await;");

        println!("\nHTTP client usage:");
        println!("let response = http_get_request(\"http://example.com\").await?;");

        println!("\nChat server usage:");
        println!("let chat = ChatServer::new();");
        println!("let mut rx = chat.add_client(\"user1\").await;");
        println!("chat.broadcast_message(\"user1\", \"Hello everyone!\").await;");

        println!("\nUDP usage:");
        println!("udp_send(\"127.0.0.1:8081\", \"Hello, UDP!\").await?;");
        println!("let (msg, addr) = udp_receive(\"127.0.0.1:8081\").await?;");

        println!("\nRate limiter usage:");
        println!("let limiter = RateLimiter::new(100, 1000, 10);");
        println!("if limiter.acquire(1).await {");
        println!("    // Execute rate-limited operation");
        println!("}");

        // This test always passes since it doesn't test anything
        assert!(true);
    }
}
