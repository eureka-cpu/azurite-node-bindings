#[path = "common/mod.rs"]
mod common;

use azurite_node_bindings::AzuriteQueue;
use std::time::Duration;
use tracing::info;

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let tmp = std::env::temp_dir().join("azurite-queue-example");
    std::fs::create_dir_all(&tmp).unwrap();

    info!("starting Azurite queue service");

    // Exercise every AzuriteQueue builder option except TLS (cert/key/pwd) and oauth.
    let azurite = AzuriteQueue::new()
        .queue_host("127.0.0.1")
        .queue_port(10001)
        // persistent workspace
        .location(tmp.to_str().unwrap())
        // behaviour flags
        .loose()
        .silent()
        .skip_api_version_check()
        .disable_telemetry()
        .disable_product_style_url()
        .start()
        .expect("failed to spawn azurite-queue");

    let pid = azurite.pid().expect("pid must be set after start");
    info!(pid, "azurite-queue process spawned");

    info!(port = 10001, "waiting for queue port");
    assert!(
        common::wait_for_port("127.0.0.1", 10001, Duration::from_secs(15)),
        "queue service did not come up on port 10001"
    );
    info!(port = 10001, "queue service ready");

    assert!(tmp.exists(), "workspace directory should exist");
    info!(path = ?tmp, "workspace directory present");

    info!("dropping handle (kills process)");
    drop(azurite);
    common::assert_process_dead(pid);
    info!(pid, "process confirmed dead");

    // Re-start with in-memory persistence and extent memory limit.
    info!("re-starting with in_memory_persistence + extent_memory_limit");
    let azurite2 = AzuriteQueue::new()
        .queue_host("127.0.0.1")
        .queue_port(10001)
        .in_memory_persistence()
        .extent_memory_limit(64)
        .loose()
        .silent()
        .skip_api_version_check()
        .disable_telemetry()
        .disable_product_style_url()
        .start()
        .expect("failed to spawn azurite-queue (in-memory)");

    let pid2 = azurite2.pid().expect("pid must be set after start");
    info!(pid = pid2, "azurite-queue (in-memory) spawned");

    assert!(
        common::wait_for_port("127.0.0.1", 10001, Duration::from_secs(15)),
        "queue service (in-memory) did not come up"
    );
    info!(port = 10001, "queue service (in-memory) ready");

    drop(azurite2);
    common::assert_process_dead(pid2);
    info!(pid = pid2, "process confirmed dead");

    std::fs::remove_dir_all(&tmp).ok();
}
