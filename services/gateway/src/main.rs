//! Command-line entry point for the LawSynth gateway.
//!
//! The single `serve` subcommand binds a listener and forwards to an upstream
//! backend using the system wall clock. TLS is never terminated here; see the
//! `tls` module for the honest boundary.

use lawsynth_gateway::{Gateway, GatewayConfig, Shutdown};
use std::net::TcpListener;

fn main() {
    if let Err(error) = run(std::env::args().skip(1).collect()) {
        eprintln!("lawsynth-gateway: {error}");
        std::process::exit(2);
    }
}

fn run(arguments: Vec<String>) -> Result<(), String> {
    let Some(command) = arguments.first().map(String::as_str) else {
        return Err(usage());
    };
    match command {
        "serve" if arguments.len() == 3 => {
            let config = GatewayConfig::new(&arguments[1], &arguments[2]);
            config.validate().map_err(|error| error.to_string())?;
            let gateway = Gateway::with_system_clock(config).map_err(|error| error.to_string())?;
            let listener = TcpListener::bind(&arguments[1])
                .map_err(|error| format!("cannot bind {}: {error}", arguments[1]))?;
            let address = listener.local_addr().map_err(|error| error.to_string())?;
            eprintln!(
                "lawsynth-gateway: serving HTTP on {address} upstream={} tls={} ({})",
                gateway.config().upstream_addr,
                gateway.config().tls_mode,
                gateway.config().tls_mode.reason()
            );
            let shutdown = Shutdown::new();
            gateway.serve(&listener, &shutdown).map_err(|error| error.to_string())
        }
        _ => Err(usage()),
    }
}

fn usage() -> String {
    "usage: lawsynth-gateway serve <listen-addr> <upstream-addr>".into()
}
