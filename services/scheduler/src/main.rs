use std::net::TcpListener;
use std::sync::{Arc, Mutex};

use lawsynth_scheduler::{Scheduler, SchedulerConfig, SchedulerServer, SchedulerTransport};
use lawsynth_store::{LocalStore, MemoryStore, ObjectStore, StoreConfig};

fn main() {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    match arguments.first().map(String::as_str) {
        // `serve <addr>` runs an in-memory scheduler; an optional `<root>`
        // durably persists checkpoints through a `LocalStore`.
        Some("serve") if arguments.len() == 2 || arguments.len() == 3 => {
            if let Err(error) = serve(&arguments[1], arguments.get(2).map(String::as_str)) {
                eprintln!("lawsynth-scheduler: {error}");
                std::process::exit(2);
            }
        }
        // No serving subcommand: keep the honest statement that executable
        // dispatch is in-process and the network surface is control-plane only.
        _ => eprintln!(
            "lawsynth-scheduler exposes {}; the HTTP surface is {} (run `serve <addr> [root]`)",
            SchedulerTransport::LocalTyped.reason(),
            SchedulerTransport::HttpControlPlane.reason(),
        ),
    }
}

fn serve(address: &str, root: Option<&str>) -> Result<(), String> {
    let config = SchedulerConfig::default();
    match root {
        Some(root) => {
            let store = LocalStore::open(root, StoreConfig::default())
                .map_err(|error| format!("cannot open store at {root}: {error}"))?;
            let scheduler = Scheduler::new(config, store).map_err(|error| error.to_string())?;
            serve_scheduler(scheduler, address, Some(root))
        }
        None => {
            let scheduler = Scheduler::new(config, MemoryStore::default())
                .map_err(|error| error.to_string())?;
            serve_scheduler(scheduler, address, None)
        }
    }
}

fn serve_scheduler<S: ObjectStore + Send + 'static>(
    scheduler: Scheduler<S>,
    address: &str,
    root: Option<&str>,
) -> Result<(), String> {
    let listener =
        TcpListener::bind(address).map_err(|error| format!("cannot bind {address}: {error}"))?;
    let local = listener.local_addr().map_err(|error| error.to_string())?;
    match root {
        Some(root) => {
            eprintln!("lawsynth-scheduler: serving control plane on {local} store={root}")
        }
        None => eprintln!("lawsynth-scheduler: serving control plane on {local} store=memory"),
    }
    let server = SchedulerServer::with_system_clock(Arc::new(Mutex::new(scheduler)));
    server.serve(&listener).map_err(|error| error.to_string())
}
