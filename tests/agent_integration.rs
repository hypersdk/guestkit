//! Agent integration tests (requires --features agent).

#![cfg(feature = "agent")]

use guestkit::agent::handler::RequestHandler;

#[test]
fn agent_ping_via_handler() {
    let handler = RequestHandler::new();
    let resp = handler.handle(br#"{"jsonrpc":"2.0","method":"guestkit.ping","id":1}"#);
    assert!(resp.result.is_some());
}

#[test]
fn agent_get_capabilities() {
    let handler = RequestHandler::new();
    let resp = handler.handle(br#"{"jsonrpc":"2.0","method":"guestkit.getCapabilities","id":2}"#);
    assert!(resp.result.is_some());
}

#[test]
fn agent_get_evidence_live() {
    let handler = RequestHandler::new();
    let resp = handler.handle(br#"{"jsonrpc":"2.0","method":"guestkit.getEvidence","id":3}"#);
    assert!(resp.result.is_some(), "{:?}", resp.error);
}

#[test]
fn agent_doctor_live() {
    let handler = RequestHandler::new();
    let resp = handler.handle(
        br#"{"jsonrpc":"2.0","method":"guestkit.doctor","params":{"target":"kvm"},"id":4}"#,
    );
    assert!(resp.result.is_some(), "{:?}", resp.error);
    let semantic = resp
        .result
        .as_ref()
        .and_then(|r| r.get("semantic"));
    assert!(semantic.is_some(), "doctor should include semantic analysis");
}

#[test]
fn agent_collect_support_bundle() {
    let handler = RequestHandler::new();
    let resp = handler.handle(
        br#"{"jsonrpc":"2.0","method":"guestkit.collectSupportBundle","id":5}"#,
    );
    assert!(resp.result.is_some(), "{:?}", resp.error);
    let result = resp.result.as_ref().expect("result");
    assert_eq!(result.get("format").and_then(|v| v.as_str()), Some("tar.zst"));
    assert_eq!(result.get("encoding").and_then(|v| v.as_str()), Some("base64"));
    let data = result.get("data").and_then(|v| v.as_str()).expect("base64 data");
    use base64::{engine::general_purpose::STANDARD, Engine};
    let bytes = STANDARD.decode(data).expect("decode bundle");
    assert!(!bytes.is_empty());
}

#[test]
#[ignore = "requires live systemd/journald; run with GUESTKIT_LIVE_AGENT_TEST=1"]
fn agent_live_systemd_events_smoke() {
    if std::env::var("GUESTKIT_LIVE_AGENT_TEST").is_err() {
        return;
    }
    let handler = RequestHandler::new();
    let resp = handler.handle(
        br#"{"jsonrpc":"2.0","method":"guestkit.getSystemdEvents","params":{"limit":5},"id":6}"#,
    );
    assert!(resp.result.is_some(), "{:?}", resp.error);
}

#[test]
fn protocol_frame_round_trip() {
    use guestkit_agent_protocol::{read_frame, write_frame};
    use std::io::Cursor;
    let payload = br#"{"jsonrpc":"2.0","method":"guestkit.ping"}"#;
    let mut buf = Vec::new();
    write_frame(&mut buf, payload).unwrap();
    let mut cursor = Cursor::new(buf);
    assert_eq!(read_frame(&mut cursor).unwrap(), payload);
}
