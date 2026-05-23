#[path = "common/mod.rs"]
mod common;

use azurite_node_bindings::Azurite;
use std::time::Duration;
use tracing::{info, warn};

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let tmp = std::env::temp_dir().join("azurite-example");
    std::fs::create_dir_all(&tmp).unwrap();
    let debug_log = tmp.join("debug.log");

    info!("starting Azurite (blob + queue + table)");

    // Exercise every builder option except TLS (cert/key/pwd) and oauth,
    // which require external infrastructure.
    let azurite = Azurite::new()
        // per-service host and port
        .blob_host("127.0.0.1")
        .blob_port(11000)
        .blob_keep_alive_timeout(60)
        .queue_host("127.0.0.1")
        .queue_port(11001)
        .queue_keep_alive_timeout(60)
        .table_host("127.0.0.1")
        .table_port(11002)
        .table_keep_alive_timeout(60)
        // storage
        .in_memory_persistence()
        .extent_memory_limit(128)
        // behaviour flags
        .loose()
        .silent()
        .skip_api_version_check()
        .disable_telemetry()
        .disable_product_style_url()
        // debug log
        .debug(debug_log.to_str().unwrap())
        .start()
        .expect("failed to spawn azurite");

    let pid = azurite.pid().expect("pid must be set after start");
    info!(pid, "azurite process spawned");

    for (name, port) in [("blob", 11000u16), ("queue", 11001), ("table", 11002)] {
        info!(service = name, port, "waiting for port");
        assert!(
            common::wait_for_port("127.0.0.1", port, Duration::from_secs(15)),
            "{name} service did not come up on port {port}"
        );
        info!(service = name, port, "ready");
    }

    if !debug_log.exists() {
        warn!(path = ?debug_log, "debug log not yet created (may appear on first request)");
    }

    info!("all services ready — dropping handle (kills process)");
    drop(azurite);
    common::assert_process_dead(pid);
    info!(pid, "process confirmed dead");

    std::fs::remove_dir_all(&tmp).ok();
}
