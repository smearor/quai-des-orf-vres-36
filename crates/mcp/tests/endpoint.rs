//! Endpoint smoke test: spawn, JSON-RPC handshake over real HTTP -
//! no GTK involved (the state arrives via `watch`).

use std::net::SocketAddr;
use std::time::Duration;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;

/// Reads the HTTP response until the SSE stream produces a data event
/// or the connection closes.
async fn read_response(stream: &mut tokio::net::TcpStream) -> String {
    let mut body = Vec::new();
    let mut buf = [0u8; 4096];
    loop {
        match stream.read(&mut buf).await {
            Ok(0) | Err(_) => break,
            Ok(n) => {
                body.extend_from_slice(&buf[..n]);
                // Headers done AND at least one SSE data event arrived.
                if body.windows(4).any(|w| w == b"\r\n\r\n") && body.windows(5).any(|w| w == b"data:") {
                    break;
                }
            }
        }
    }
    String::from_utf8_lossy(&body).to_string()
}

/// One raw HTTP/1.1 JSON-RPC POST to `/mcp`.
async fn post(addr: SocketAddr, body: &str) -> String {
    let request = format!(
        "POST /mcp HTTP/1.1\r\nHost: localhost:{}\r\nContent-Type: application/json\r\nAccept: application/json, text/event-stream\r\nContent-Length: {}\r\n\r\n{}",
        addr.port(),
        body.len(),
        body,
    );
    let Ok(mut stream) = tokio::net::TcpStream::connect(addr).await else {
        return "connect failed".to_string();
    };
    let _ = stream.write_all(request.as_bytes()).await;
    let _ = stream.flush().await;
    match tokio::time::timeout(Duration::from_secs(3), read_response(&mut stream)).await {
        Ok(response) => response,
        Err(_) => "no response".to_string(),
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn serves_initialize_handshake() {
    let addr: SocketAddr = "127.0.0.1:19877".parse().unwrap_or_else(|_| unreachable!());
    let (tx, rx) = tokio::sync::watch::channel(serde_json::json!({"anchor": {"detected": false}}));
    qdo36_mcp::spawn_http(rx, addr);

    // The endpoint runs on its own thread - give the bind a moment.
    tokio::time::sleep(Duration::from_millis(300)).await;
    let _ = tx.send(serde_json::json!({"anchor": {"detected": true}}));

    let response = post(
        addr,
        r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-03-26","capabilities":{},"clientInfo":{"name":"smoke","version":"0"}}}"#,
    )
    .await;
    assert!(response.contains("200"), "initialize failed:\n{response}");
    assert!(response.contains("protocolVersion"), "missing protocol version:\n{response}");
    assert!(response.contains("mcp-session-id"), "missing session header:\n{response}");
}
