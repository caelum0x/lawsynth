//! Integration tests for the server entrypoint behavior exercised by `main`.
//!
//! `main` builds a [`SchedulerServer`] with the system clock and serves the
//! control plane, and otherwise prints the honest transport surfaces. These tests
//! drive that same serve path over a real socket and assert the transport
//! messaging `main` emits.

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use lawsynth_scheduler::{Scheduler, SchedulerConfig, SchedulerServer, SchedulerTransport};
use lawsynth_store::MemoryStore;

fn make_scheduler() -> Scheduler<MemoryStore> {
    let config = SchedulerConfig::new(8, 2, Duration::from_millis(50), 8192).unwrap();
    Scheduler::new(config, MemoryStore::default()).unwrap()
}

#[test]
fn system_clock_server_serves_health_over_a_socket() {
    let scheduler = Arc::new(Mutex::new(make_scheduler()));
    let server = SchedulerServer::with_system_clock(Arc::clone(&scheduler));

    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let serving = thread::spawn(move || {
        let _ = server.serve(&listener);
    });

    let mut stream = TcpStream::connect(address).unwrap();
    stream
        .write_all(b"GET /health HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n")
        .unwrap();
    stream.flush().unwrap();
    let mut response = String::new();
    stream.read_to_string(&mut response).unwrap();

    assert!(response.starts_with("HTTP/1.1 200 OK\r\n"), "unexpected status line: {response}");
    assert!(response.contains("application/json"));
    assert!(response.contains("\"queued_count\":0"));
    assert!(response.contains("\"ready\":true"));
    assert!(response.contains("\"metrics\":"));

    drop(stream);
    drop(serving);
}

#[test]
fn transport_surfaces_reported_by_main_are_honest() {
    // `main` prints these two reasons when no serve subcommand is given.
    assert!(SchedulerTransport::LocalTyped.reason().contains("in-process typed dispatch"));
    assert!(
        SchedulerTransport::HttpControlPlane
            .reason()
            .contains("executable job dispatch stays in-process")
    );
    // The control plane is available; the broker seam is not.
    assert!(SchedulerTransport::HttpControlPlane.is_available());
    assert!(!SchedulerTransport::BrokerNotLinked.is_available());
}
