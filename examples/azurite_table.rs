#[path = "common/mod.rs"]
mod common;

use azurite_node_bindings::AzuriteTable;
use std::time::Duration;
use tracing::info;

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let tmp = std::env::temp_dir().join("azurite-table-example");
    std::fs::create_dir_all(&tmp).unwrap();

    info!("starting Azurite table service");

    // Exercise every AzuriteTable builder option except TLS (cert/key/pwd) and oauth.
    // Note: AzuriteTable has no extentMemoryLimit option.
    let azurite = AzuriteTable::new()
        .table_host("127.0.0.1")
        .table_port(10002)
        .table_keep_alive_timeout(60)
        // persistent workspace
        .location(tmp.to_str().unwrap())
        // behaviour flags
        .loose()
        .silent()
        .skip_api_version_check()
        .disable_telemetry()
        .disable_product_style_url()
        .start()
        .expect("failed to spawn azurite-table");

    let pid = azurite.pid().expect("pid must be set after start");
    info!(pid, "azurite-table process spawned");

    info!(port = 10002, "waiting for table port");
    assert!(
        common::wait_for_port("127.0.0.1", 10002, Duration::from_secs(15)),
        "table service did not come up on port 10002"
    );
    info!(port = 10002, "table service ready");

    assert!(tmp.exists(), "workspace directory should exist");
    info!(path = ?tmp, "workspace directory present");

    info!("dropping handle (kills process)");
    drop(azurite);
    common::assert_process_dead(pid);
    info!(pid, "process confirmed dead");

    // Re-start with in-memory persistence to exercise that option.
    info!("re-starting with in_memory_persistence");
    let azurite2 = AzuriteTable::new()
        .table_host("127.0.0.1")
        .table_port(10002)
        .table_keep_alive_timeout(30)
        .in_memory_persistence()
        .loose()
        .silent()
        .skip_api_version_check()
        .disable_telemetry()
        .disable_product_style_url()
        .start()
        .expect("failed to spawn azurite-table (in-memory)");

    let pid2 = azurite2.pid().expect("pid must be set after start");
    info!(pid = pid2, "azurite-table (in-memory) spawned");

    assert!(
        common::wait_for_port("127.0.0.1", 10002, Duration::from_secs(15)),
        "table service (in-memory) did not come up"
    );
    info!(port = 10002, "table service (in-memory) ready");

    drop(azurite2);
    common::assert_process_dead(pid2);
    info!(pid = pid2, "process confirmed dead");

    std::fs::remove_dir_all(&tmp).ok();
}
