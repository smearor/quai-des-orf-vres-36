//! Endpoint smoke test: spawn, handshake, and a `tools/call` over
//! real HTTP - no GTK involved (the state arrives via `watch`).

use qdo36_mcp::CommandResult;
use qdo36_mcp::GameRequest;
use qdo36_mcp::StateSnapshot;
use qdo36_mcp::spawn_http;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::time::sleep;
use tokio::time::timeout;

/// Reads the HTTP response: until the connection closes, the JSON-RPC
/// result arrived, or the stream goes idle for 250ms.
async fn read_response(stream: &mut TcpStream) -> String {
    let mut body = Vec::new();
    let mut buf = [0u8; 4096];
    loop {
        match timeout(Duration::from_millis(250), stream.read(&mut buf)).await {
            Ok(Ok(0)) | Err(_) => break,
            Ok(Err(_)) => break,
            Ok(Ok(n)) => {
                body.extend_from_slice(&buf[..n]);
                // A `data:` frame containing the JSON-RPC envelope is the
                // answer; priming SSE frames carry only `id:`/`retry:`.
                if body.windows(4).any(|w| w == b"\r\n\r\n") && String::from_utf8_lossy(&body).contains("data: {\"jsonrpc\":") {
                    break;
                }
            }
        }
    }
    String::from_utf8_lossy(&body).to_string()
}

/// One raw HTTP/1.1 JSON-RPC POST to `/mcp`.
async fn post(addr: SocketAddr, session: Option<&str>, body: &str) -> String {
    let session_header = session.map(|id| format!("Mcp-Session-Id: {id}\r\n")).unwrap_or_default();
    let request = format!(
        "POST /mcp HTTP/1.1\r\nHost: localhost:{}\r\nContent-Type: application/json\r\nAccept: application/json, text/event-stream\r\n{session_header}Content-Length: {}\r\n\r\n{}",
        addr.port(),
        body.len(),
        body,
    );
    let Ok(mut stream) = tokio::net::TcpStream::connect(addr).await else {
        return "connect failed".to_string();
    };
    let _ = stream.write_all(request.as_bytes()).await;
    let _ = stream.flush().await;
    match timeout(Duration::from_secs(3), read_response(&mut stream)).await {
        Ok(response) => response,
        Err(_) => "no response".to_string(),
    }
}

/// The `mcp-session-id` header value out of a raw response.
fn session_id(response: &str) -> String {
    response
        .lines()
        .find_map(|line| line.strip_prefix("mcp-session-id: "))
        .map(|id| id.trim().to_string())
        .unwrap_or_default()
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn serves_tools_over_streamable_http() {
    let addr: SocketAddr = "127.0.0.1:19877".parse().unwrap_or_else(|_| unreachable!());
    let (tx, rx) = tokio::sync::watch::channel(StateSnapshot::default());
    let (command_tx, command_rx) = async_channel::unbounded::<GameRequest>();
    spawn_http(command_tx, rx, addr);
    // Stand-in for the app-side game host: reply `GameBegan` to the
    // first command so the tools/call round-trip is exercised.
    tokio::spawn(async move {
        while let Ok(request) = command_rx.recv().await {
            let _ = request.reply.send(Ok(CommandResult::GameBegan));
        }
    });

    // The endpoint runs on its own thread - give the bind a moment,
    // then publish state the tools will answer with.
    sleep(Duration::from_millis(300)).await;
    tx.send_modify(|doc| doc.anchor.detected = true);

    let response = post(
        addr,
        None,
        r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-03-26","capabilities":{},"clientInfo":{"name":"smoke","version":"0"}}}"#,
    )
    .await;
    assert!(response.contains("200"), "initialize failed:\n{response}");
    assert!(response.contains("protocolVersion"), "missing protocol version:\n{response}");
    let session = session_id(&response);
    assert!(!session.is_empty(), "missing session header:\n{response}");

    // Complete the handshake, then list and call the introspection tools.
    let _ = post(addr, Some(&session), r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#).await;

    let response = post(addr, Some(&session), r#"{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}"#).await;
    assert!(response.contains("anchor_state") && response.contains("map_info"), "tools/list missing tools:\n{response}");

    let response = post(
        addr,
        Some(&session),
        r#"{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"anchor_state","arguments":{}}}"#,
    )
    .await;
    assert!(response.contains("detected"), "anchor_state missing payload:\n{response}");
    assert!(response.contains("true"), "anchor_state did not see the published flag:\n{response}");

    // Command round-trip: `begin_game` travels the channel and gets
    // the stand-in host's `GameBegan` reply.
    let response = post(
        addr,
        Some(&session),
        r#"{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"begin_game","arguments":{}}}"#,
    )
    .await;
    assert!(
        response.contains("game_began") || response.contains("GameBegan"),
        "command channel did not answer:\n{response}"
    );
}
