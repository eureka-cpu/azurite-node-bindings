#[path = "common/mod.rs"]
mod common;

use azurite_node_bindings::AzuriteBlob;
use std::time::Duration;
use tracing::info;

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let tmp = std::env::temp_dir().join("azurite-blob-example");
    std::fs::create_dir_all(&tmp).unwrap();

    info!("starting Azurite blob service");

    // Exercise every AzuriteBlob builder option except TLS (cert/key/pwd) and oauth.
    let azurite = AzuriteBlob::new()
        .blob_host("127.0.0.1")
        .blob_port(10000)
        .blob_keep_alive_timeout(60)
        // persistent workspace (on-disk, not in-memory, so location is used)
        .location(tmp.to_str().unwrap())
        // behaviour flags
        .loose()
        .silent()
        .skip_api_version_check()
        .disable_telemetry()
        .disable_product_style_url()
        .start()
        .expect("failed to spawn azurite-blob");

    let pid = azurite.pid().expect("pid must be set after start");
    info!(pid, "azurite-blob process spawned");

    info!(port = 10000, "waiting for blob port");
    assert!(
        common::wait_for_port("127.0.0.1", 10000, Duration::from_secs(15)),
        "blob service did not come up on port 10000"
    );
    info!(port = 10000, "blob service ready");

    // Verify the workspace directory was used
    assert!(tmp.exists(), "workspace directory should exist");
    info!(path = ?tmp, "workspace directory present");

    info!("dropping handle (kills process)");
    drop(azurite);
    common::assert_process_dead(pid);
    info!(pid, "process confirmed dead");

    // Re-start with in-memory persistence and extent memory limit to exercise those options.
    info!("re-starting with in_memory_persistence + extent_memory_limit");
    let azurite2 = AzuriteBlob::new()
        .blob_host("127.0.0.1")
        .blob_port(10000)
        .blob_keep_alive_timeout(30)
        .in_memory_persistence()
        .extent_memory_limit(64)
        .loose()
        .silent()
        .skip_api_version_check()
        .disable_telemetry()
        .disable_product_style_url()
        .start()
        .expect("failed to spawn azurite-blob (in-memory)");

    let pid2 = azurite2.pid().expect("pid must be set after start");
    info!(pid = pid2, "azurite-blob (in-memory) spawned");

    assert!(
        common::wait_for_port("127.0.0.1", 10000, Duration::from_secs(15)),
        "blob service (in-memory) did not come up"
    );
    info!(port = 10000, "blob service (in-memory) ready");

    drop(azurite2);
    common::assert_process_dead(pid2);
    info!(pid = pid2, "process confirmed dead");

    std::fs::remove_dir_all(&tmp).ok();
}
