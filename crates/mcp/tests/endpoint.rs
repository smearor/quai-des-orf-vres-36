//! Endpoint smoke test: spawn, handshake, and `tools/call`s over
//! real HTTP - no GTK involved (the state arrives via `watch`,
//! commands travel the request channel).

use qdo36_host::GameHost;
use qdo36_map::GameMap;
use qdo36_mcp::CommandResult;
use qdo36_mcp::GameRequest;
use qdo36_mcp::StateSnapshot;
use qdo36_mcp::spawn_http;
use rand::SeedableRng;
use rand::rngs::StdRng;
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
/// The tool reply's JSON payload: HTTP body → `data:`/`{` line →
/// `content[0].text` → the serialized `CommandResult` JSON.
fn tool_payload(response: &str) -> serde_json::Value {
    let body = response.split("\r\n\r\n").nth(1).unwrap_or_default();
    let payload_line = body
        .lines()
        .filter_map(|line| line.strip_prefix("data: ").or_else(|| line.starts_with('{').then_some(line)))
        .find(|line| !line.trim().is_empty())
        .unwrap_or_default();
    let json: serde_json::Value = serde_json::from_str(payload_line).unwrap_or_else(|_| serde_json::json!({}));
    let text = json["result"]["content"][0]["text"].as_str().unwrap_or_default();
    serde_json::from_str(text).unwrap_or_else(|_| serde_json::json!({}))
}

/// Fires a `tools/call` and returns the parsed reply payload.
async fn call(addr: SocketAddr, session: &str, id: u32, name: &str, args: serde_json::Value) -> serde_json::Value {
    let body = serde_json::json!({"jsonrpc":"2.0","id":id,"method":"tools/call","params":{"name":name,"arguments":args}}).to_string();
    let response = post(addr, Some(session), &body).await;
    assert!(!response.contains("\"isError\":true"), "{name} failed:\n{response}");
    tool_payload(&response)
}

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

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn plays_a_move_end_to_end() {
    let addr: SocketAddr = "127.0.0.1:19878".parse().unwrap_or_else(|_| unreachable!());
    let (_state_tx, state_rx) = tokio::sync::watch::channel(StateSnapshot::default());
    let (command_tx, command_rx) = async_channel::unbounded::<GameRequest>();
    spawn_http(command_tx, state_rx, addr);

    let map = GameMap::bundled().unwrap_or_else(|_| panic!("bundled map"));
    tokio::spawn(async move {
        let mut host = GameHost::with_rng(StdRng::seed_from_u64(36));
        host.set_map(std::sync::Arc::new(map));
        while let Ok(request) = command_rx.recv().await {
            let _ = request.reply.send(host.handle(&request.command));
        }
    });

    sleep(Duration::from_millis(300)).await;
    let response = post(
        addr,
        None,
        r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-03-26","capabilities":{},"clientInfo":{"name":"e2e","version":"0"}}}"#,
    )
    .await;
    let session = session_id(&response);
    let _ = post(addr, Some(&session), r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#).await;

    // Deal → begin → join as the fugitive.
    call(addr, &session, 2, "new_game", serde_json::json!({})).await;
    call(addr, &session, 3, "begin_game", serde_json::json!({})).await;
    let joined = call(addr, &session, 4, "join_game", serde_json::json!({"claim": "fugitive"})).await;
    let token = joined["joined"]["token"].as_str().unwrap_or_default().to_string();
    assert!(token.len() == 16, "join_game should return a hex token:\n{joined}");

    // The fugitive token sees his position; a legal hop is his
    // neighbour via an affordable transport (taxi covers most edges;
    // pick the first legal hop outright).
    let view = call(addr, &session, 5, "game_state", serde_json::json!({"token": token})).await;
    assert!(view["state"]["fugitive_pos"].is_u64(), "fugitive token should see the position:\n{view}");
    let legal = call(addr, &session, 6, "legal_moves", serde_json::json!({"token": token})).await;
    let hop = &legal["legal"][0];
    assert!(hop.is_object(), "legal_moves should list hops:\n{legal}");

    // Play the hop through request_move; the black ticket covers any
    // transport (the first legal hop could be a batobus edge).
    let args = serde_json::json!({
        "token": token,
        "ticket": "black",
        "from": view["state"]["fugitive_pos"],
        "transport": hop["transport"],
        "to": hop["to"],
    });
    let moved = call(addr, &session, 7, "request_move", args).await;
    assert!(moved.get("moved").is_some(), "request_move should apply:\n{moved}");
}
