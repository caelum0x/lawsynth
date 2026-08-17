//! Shared test scaffolding: mock upstreams, deterministic gateways, and a tiny
//! blocking HTTP client. Not every test binary uses every helper, so dead-code
//! warnings are silenced for this shared module.
#![allow(dead_code)]

use lawsynth_gateway::{Clock, Gateway, GatewayConfig};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;
use std::thread;

/// A clock pinned to `now`, so rate-limit windows are fully controlled.
pub fn fixed_clock(now: u64) -> Clock {
    Arc::new(move || now)
}

/// A gateway pointed at `upstream`, with a fixed clock and default limits.
pub fn gateway(upstream: &str, now: u64) -> Gateway {
    Gateway::new(GatewayConfig::new("127.0.0.1:0", upstream), fixed_clock(now)).unwrap()
}

/// A gateway with a custom base config and fixed clock.
pub fn gateway_with(config: GatewayConfig, now: u64) -> Gateway {
    Gateway::new(config, fixed_clock(now)).unwrap()
}

/// Spawns a one-shot mock upstream that drains the request (honouring
/// `Content-Length`) and then writes `canned`. The join handle yields the raw
/// bytes the upstream received, so tests can assert on the forwarded request.
pub fn spawn_upstream(canned: Vec<u8>) -> (String, thread::JoinHandle<Vec<u8>>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap().to_string();
    let handle = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let received = drain_request(&mut stream);
        stream.write_all(&canned).unwrap();
        stream.flush().unwrap();
        received
    });
    (address, handle)
}

/// Reads a full HTTP request (head plus any `Content-Length` body) from a stream.
fn drain_request(stream: &mut TcpStream) -> Vec<u8> {
    let mut buffer = Vec::new();
    let mut chunk = [0u8; 4096];
    let mut header_end = None;
    while header_end.is_none() {
        let read = stream.read(&mut chunk).unwrap();
        if read == 0 {
            return buffer;
        }
        buffer.extend_from_slice(&chunk[..read]);
        header_end = find_subsequence(&buffer, b"\r\n\r\n").map(|i| i + 4);
    }
    let header_end = header_end.unwrap();
    let head = String::from_utf8_lossy(&buffer[..header_end]).to_ascii_lowercase();
    let content_length = head
        .lines()
        .find_map(|line| line.strip_prefix("content-length:"))
        .and_then(|value| value.trim().parse::<usize>().ok())
        .unwrap_or(0);

    let have = buffer.len() - header_end;
    let mut remaining = content_length.saturating_sub(have);
    while remaining > 0 {
        let read = stream.read(&mut chunk).unwrap();
        if read == 0 {
            break;
        }
        buffer.extend_from_slice(&chunk[..read]);
        remaining = remaining.saturating_sub(read);
    }
    buffer
}

fn find_subsequence(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack.windows(needle.len()).position(|window| window == needle)
}

/// Runs a full request/response round-trip against a listening gateway address.
pub fn round_trip(address: &str, request: &[u8]) -> String {
    let mut stream = TcpStream::connect(address).unwrap();
    stream.write_all(request).unwrap();
    stream.flush().unwrap();
    let mut response = String::new();
    stream.read_to_string(&mut response).unwrap();
    response
}

/// Binds an ephemeral listener and serves `gateway` on a background thread,
/// returning the bound address and a handle that stops the accept loop on drop.
pub fn serve(gateway: Gateway) -> (String, lawsynth_gateway::ShutdownHandle) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap().to_string();
    let shutdown = lawsynth_gateway::Shutdown::new();
    let handle = shutdown.handle();
    thread::spawn(move || {
        let _ = gateway.serve(&listener, &shutdown);
    });
    (address, handle)
}
