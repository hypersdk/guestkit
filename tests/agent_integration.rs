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
