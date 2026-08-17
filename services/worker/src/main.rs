use std::net::TcpListener;
use std::sync::Arc;

use lawsynth_store::{LocalStore, StoreConfig};
use lawsynth_worker::{TransportSurface, Worker, WorkerConfig, WorkerServer};

fn main() {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    match arguments.first().map(String::as_str) {
        Some("serve") if arguments.len() == 3 => {
            if let Err(error) = serve(&arguments[1], &arguments[2]) {
                eprintln!("lawsynth-worker: {error}");
                std::process::exit(2);
            }
        }
        None => {
            // Honest default: the executable job surface is in-process and typed;
            // only the read-only HTTP status transport can be served over a socket.
            eprintln!(
                "lawsynth-worker exposes {}; the HTTP transport {}",
                TransportSurface::LocalDirect.reason(),
                TransportSurface::HttpStatus.reason()
            );
            eprintln!("usage: lawsynth-worker serve <store-root> <addr>");
        }
        _ => {
            eprintln!("usage: lawsynth-worker serve <store-root> <addr>");
            std::process::exit(2);
        }
    }
}

/// Opens a durable store rooted at `root` and serves the read-only HTTP status
/// transport on `addr`, using the system wall clock for reported timestamps.
fn serve(root: &str, addr: &str) -> Result<(), String> {
    let store =
        LocalStore::open(root, StoreConfig::default()).map_err(|error| error.to_string())?;
    let worker = Worker::new(WorkerConfig::default(), store).map_err(|error| error.to_string())?;
    let listener =
        TcpListener::bind(addr).map_err(|error| format!("cannot bind {addr}: {error}"))?;
    let address = listener.local_addr().map_err(|error| error.to_string())?;
    eprintln!("lawsynth-worker: serving HTTP status on {address} store-root={root}");
    WorkerServer::with_system_clock(Arc::new(worker))
        .serve(&listener)
        .map_err(|error| error.to_string())
}
