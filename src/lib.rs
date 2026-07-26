//! # Azurite Node Bindings
//!
//! Rust bindings for the Azurite Azure storage emulation node(s).
//!
//! This is a wrapper around `std::process::Command` which uses the `azurite`
//! command line tool directly, unlike the [`testcontainer` crate flavor of `azurite`](https://docs.rs/testcontainers-modules/latest/testcontainers_modules/azurite/struct.Azurite.html)
//! which requires `docker`.
//!
//! Each process is killed on `drop`.
//!
//! The toplevel `azurite` command can be ran using [`Azurite`], or each node
//! can be ran individually using [`AzuriteBlob`], [`AzuriteTable`] and [`AzuriteQueue`].
//!
//! ## Example Usage
//!
//! The following examples runs the toplevel `azurite` command and asserts all resource nodes
//! can be reached, and that their process is killed on `drop`.
//!
//! See the [`examples`](https://github.com/eureka-cpu/azurite-node-bindings/blob/master/examples) directory of the repository for resource specific usage.
//!
//! ```rust,ignore
//! use azurite_node_bindings::Azurite;
//! use std::time::Duration;
//! use tracing::{info, warn};
//!
//! mod common {
//!     use std::net::{SocketAddr, TcpStream};
//!     use std::path::Path;
//!     use std::time::{Duration, Instant};
//!
//!     /// Poll `host:port` until a TCP connection succeeds or `timeout` elapses.
//!     pub fn wait_for_port(host: &str, port: u16, timeout: Duration) -> bool {
//!         let addr: SocketAddr = format!("{host}:{port}").parse().unwrap();
//!         let deadline = Instant::now() + timeout;
//!         while Instant::now() < deadline {
//!             if TcpStream::connect_timeout(&addr, Duration::from_millis(100)).is_ok() {
//!                 return true;
//!             }
//!             std::thread::sleep(Duration::from_millis(200));
//!         }
//!         false
//!     }
//!
//!     /// Assert that a process with the given PID is no longer alive.
//!     /// Checks `/proc/<pid>` on Linux.
//!     pub fn assert_process_dead(pid: u32) {
//!         std::thread::sleep(Duration::from_millis(200));
//!         assert!(
//!             !Path::new(&format!("/proc/{pid}")).exists(),
//!             "process {pid} should have been killed on drop"
//!         );
//!     }
//! }
//!
//! fn main() {
//!     tracing_subscriber::fmt()
//!         .with_env_filter(
//!             tracing_subscriber::EnvFilter::try_from_default_env()
//!                 .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
//!         )
//!         .init();
//!
//!     let tmp = std::env::temp_dir().join("azurite-example");
//!     std::fs::create_dir_all(&tmp).unwrap();
//!     let debug_log = tmp.join("debug.log");
//!
//!     info!("starting Azurite (blob + queue + table)");
//!
//!     // Exercise every builder option except TLS (cert/key/pwd) and oauth,
//!     // which require external infrastructure.
//!     let azurite = Azurite::new()
//!         // per-service host and port
//!         .blob_host("127.0.0.1")
//!         .blob_port(11000)
//!         .blob_keep_alive_timeout(60)
//!         .queue_host("127.0.0.1")
//!         .queue_port(11001)
//!         .queue_keep_alive_timeout(60)
//!         .table_host("127.0.0.1")
//!         .table_port(11002)
//!         .table_keep_alive_timeout(60)
//!         // storage
//!         .in_memory_persistence()
//!         .extent_memory_limit(128)
//!         // behaviour flags
//!         .loose()
//!         .silent()
//!         .skip_api_version_check()
//!         .disable_telemetry()
//!         .disable_product_style_url()
//!         // debug log
//!         .debug(debug_log.to_str().unwrap())
//!         .start()
//!         .expect("failed to spawn azurite");
//!
//!     let pid = azurite.pid();
//!     info!(pid, "azurite process spawned");
//!
//!     for (name, port) in [("blob", 11000u16), ("queue", 11001), ("table", 11002)] {
//!         info!(service = name, port, "waiting for port");
//!         assert!(
//!             common::wait_for_port("127.0.0.1", port, Duration::from_secs(15)),
//!             "{name} service did not come up on port {port}"
//!         );
//!         info!(service = name, port, "ready");
//!     }
//!
//!     if !debug_log.exists() {
//!         warn!(path = ?debug_log, "debug log not yet created (may appear on first request)");
//!     }
//!
//!     info!("all services ready — dropping handle (kills process)");
//!     drop(azurite);
//!     common::assert_process_dead(pid);
//!     info!(pid, "process confirmed dead");
//!
//!     std::fs::remove_dir_all(&tmp).ok();
//! }
//! ```

use std::io;
use std::marker::PhantomData;
use std::process::{Child, Command};

/// Marker type for the builder state — builder methods are available, the process has not started.
#[doc(hidden)]
pub struct Builder;

/// Runs all three Azurite services (blob, table, queue) in a single process.
///
/// The type parameter `State` tracks whether the process has been started:
/// - [`Azurite<Builder>`] — builder methods available, process not yet running.
/// - [`Azurite<()>`] — process is running, `pid()` and similar methods available.
///
/// # Example
///
/// ```no_run
/// # use azurite_node_bindings::Azurite;
/// let azurite = Azurite::new()
///     .blob_port(10000)
///     .queue_port(10001)
///     .table_port(10002)
///     .silent()
///     .start()
///     .unwrap();
/// // process is killed when `azurite` is dropped
/// ```
pub struct Azurite<State = Builder> {
    blob_host: Option<String>,
    blob_keep_alive_timeout: Option<u64>,
    blob_port: Option<u16>,
    queue_host: Option<String>,
    queue_keep_alive_timeout: Option<u64>,
    queue_port: Option<u16>,
    table_host: Option<String>,
    table_keep_alive_timeout: Option<u64>,
    table_port: Option<u16>,
    cert: Option<String>,
    debug: Option<String>,
    disable_product_style_url: bool,
    disable_telemetry: bool,
    extent_memory_limit: Option<i64>,
    in_memory_persistence: bool,
    key: Option<String>,
    location: Option<String>,
    loose: bool,
    oauth: Option<String>,
    pwd: Option<String>,
    silent: bool,
    skip_api_version_check: bool,
    pid: Option<u32>,
    child: Option<Child>,
    _state: PhantomData<State>,
}

impl Azurite<Builder> {
    pub fn new() -> Self {
        Self {
            blob_host: None,
            blob_keep_alive_timeout: None,
            blob_port: None,
            queue_host: None,
            queue_keep_alive_timeout: None,
            queue_port: None,
            table_host: None,
            table_keep_alive_timeout: None,
            table_port: None,
            cert: None,
            debug: None,
            disable_product_style_url: false,
            disable_telemetry: false,
            extent_memory_limit: None,
            in_memory_persistence: false,
            key: None,
            location: None,
            loose: false,
            oauth: None,
            pwd: None,
            silent: false,
            skip_api_version_check: false,
            pid: None,
            child: None,
            _state: PhantomData,
        }
    }

    pub fn blob_host(mut self, v: impl Into<String>) -> Self {
        self.blob_host = Some(v.into());
        self
    }
    pub fn blob_keep_alive_timeout(mut self, v: u64) -> Self {
        self.blob_keep_alive_timeout = Some(v);
        self
    }
    pub fn blob_port(mut self, v: u16) -> Self {
        self.blob_port = Some(v);
        self
    }
    pub fn queue_host(mut self, v: impl Into<String>) -> Self {
        self.queue_host = Some(v.into());
        self
    }
    pub fn queue_keep_alive_timeout(mut self, v: u64) -> Self {
        self.queue_keep_alive_timeout = Some(v);
        self
    }
    pub fn queue_port(mut self, v: u16) -> Self {
        self.queue_port = Some(v);
        self
    }
    pub fn table_host(mut self, v: impl Into<String>) -> Self {
        self.table_host = Some(v.into());
        self
    }
    pub fn table_keep_alive_timeout(mut self, v: u64) -> Self {
        self.table_keep_alive_timeout = Some(v);
        self
    }
    pub fn table_port(mut self, v: u16) -> Self {
        self.table_port = Some(v);
        self
    }
    pub fn cert(mut self, v: impl Into<String>) -> Self {
        self.cert = Some(v.into());
        self
    }
    pub fn debug(mut self, path: impl Into<String>) -> Self {
        self.debug = Some(path.into());
        self
    }
    pub fn disable_product_style_url(mut self) -> Self {
        self.disable_product_style_url = true;
        self
    }
    pub fn disable_telemetry(mut self) -> Self {
        self.disable_telemetry = true;
        self
    }
    pub fn extent_memory_limit(mut self, mb: i64) -> Self {
        self.extent_memory_limit = Some(mb);
        self
    }
    pub fn in_memory_persistence(mut self) -> Self {
        self.in_memory_persistence = true;
        self
    }
    pub fn key(mut self, v: impl Into<String>) -> Self {
        self.key = Some(v.into());
        self
    }
    pub fn location(mut self, v: impl Into<String>) -> Self {
        self.location = Some(v.into());
        self
    }
    pub fn loose(mut self) -> Self {
        self.loose = true;
        self
    }
    pub fn oauth(mut self, level: impl Into<String>) -> Self {
        self.oauth = Some(level.into());
        self
    }
    pub fn pwd(mut self, v: impl Into<String>) -> Self {
        self.pwd = Some(v.into());
        self
    }
    pub fn silent(mut self) -> Self {
        self.silent = true;
        self
    }
    pub fn skip_api_version_check(mut self) -> Self {
        self.skip_api_version_check = true;
        self
    }

    /// Spawn the `azurite` process with the configured options.
    pub fn start(mut self) -> io::Result<Azurite<()>> {
        let mut cmd = Command::new("azurite");
        if let Some(ref v) = self.blob_host {
            cmd.args(["--blobHost", v]);
        }
        if let Some(v) = self.blob_keep_alive_timeout {
            cmd.args(["--blobKeepAliveTimeout", &v.to_string()]);
        }
        if let Some(v) = self.blob_port {
            cmd.args(["--blobPort", &v.to_string()]);
        }
        if let Some(ref v) = self.queue_host {
            cmd.args(["--queueHost", v]);
        }
        if let Some(v) = self.queue_keep_alive_timeout {
            cmd.args(["--queueKeepAliveTimeout", &v.to_string()]);
        }
        if let Some(v) = self.queue_port {
            cmd.args(["--queuePort", &v.to_string()]);
        }
        if let Some(ref v) = self.table_host {
            cmd.args(["--tableHost", v]);
        }
        if let Some(v) = self.table_keep_alive_timeout {
            cmd.args(["--tableKeepAliveTimeout", &v.to_string()]);
        }
        if let Some(v) = self.table_port {
            cmd.args(["--tablePort", &v.to_string()]);
        }
        if let Some(ref v) = self.cert {
            cmd.args(["--cert", v]);
        }
        if let Some(ref v) = self.debug {
            cmd.args(["--debug", v]);
        }
        if self.disable_product_style_url {
            cmd.arg("--disableProductStyleUrl");
        }
        if self.disable_telemetry {
            cmd.arg("--disableTelemetry");
        }
        if let Some(v) = self.extent_memory_limit {
            cmd.args(["--extentMemoryLimit", &v.to_string()]);
        }
        if self.in_memory_persistence {
            cmd.arg("--inMemoryPersistence");
        }
        if let Some(ref v) = self.key {
            cmd.args(["--key", v]);
        }
        if let Some(ref v) = self.location {
            cmd.args(["--location", v]);
        }
        if self.loose {
            cmd.arg("--loose");
        }
        if let Some(ref v) = self.oauth {
            cmd.args(["--oauth", v]);
        }
        if let Some(ref v) = self.pwd {
            cmd.args(["--pwd", v]);
        }
        if self.silent {
            cmd.arg("--silent");
        }
        if self.skip_api_version_check {
            cmd.arg("--skipApiVersionCheck");
        }
        let child = cmd.spawn()?;
        Ok(Azurite {
            blob_host: self.blob_host.take(),
            blob_keep_alive_timeout: self.blob_keep_alive_timeout,
            blob_port: self.blob_port,
            queue_host: self.queue_host.take(),
            queue_keep_alive_timeout: self.queue_keep_alive_timeout,
            queue_port: self.queue_port,
            table_host: self.table_host.take(),
            table_keep_alive_timeout: self.table_keep_alive_timeout,
            table_port: self.table_port,
            cert: self.cert.take(),
            debug: self.debug.take(),
            disable_product_style_url: self.disable_product_style_url,
            disable_telemetry: self.disable_telemetry,
            extent_memory_limit: self.extent_memory_limit,
            in_memory_persistence: self.in_memory_persistence,
            key: self.key.take(),
            location: self.location.take(),
            loose: self.loose,
            oauth: self.oauth.take(),
            pwd: self.pwd.take(),
            silent: self.silent,
            skip_api_version_check: self.skip_api_version_check,
            pid: Some(child.id()),
            child: Some(child),
            _state: PhantomData,
        })
    }
}

impl Azurite<()> {
    /// Returns the PID of the spawned process.
    pub fn pid(&self) -> u32 {
        self.pid.expect("pid is set after start")
    }
}

impl Default for Azurite<Builder> {
    fn default() -> Self {
        Self::new()
    }
}

impl<State> Drop for Azurite<State> {
    fn drop(&mut self) {
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

/// Runs only the Azurite blob storage service.
///
/// The type parameter `State` tracks whether the process has been started:
/// - [`AzuriteBlob<Builder>`] — builder methods available, process not yet running.
/// - [`AzuriteBlob<()>`] — process is running, `pid()` and similar methods available.
///
/// # Example
///
/// ```no_run
/// # use azurite_node_bindings::AzuriteBlob;
/// let azurite_blob = AzuriteBlob::new()
///     .blob_port(10000)
///     .in_memory_persistence()
///     .start()
///     .unwrap();
/// ```
pub struct AzuriteBlob<State = Builder> {
    blob_host: Option<String>,
    blob_keep_alive_timeout: Option<u64>,
    blob_port: Option<u16>,
    cert: Option<String>,
    debug: Option<String>,
    disable_product_style_url: bool,
    disable_telemetry: bool,
    extent_memory_limit: Option<i64>,
    in_memory_persistence: bool,
    key: Option<String>,
    location: Option<String>,
    loose: bool,
    oauth: Option<String>,
    pwd: Option<String>,
    silent: bool,
    skip_api_version_check: bool,
    pid: Option<u32>,
    child: Option<Child>,
    _state: PhantomData<State>,
}

impl AzuriteBlob<Builder> {
    pub fn new() -> Self {
        Self {
            blob_host: None,
            blob_keep_alive_timeout: None,
            blob_port: None,
            cert: None,
            debug: None,
            disable_product_style_url: false,
            disable_telemetry: false,
            extent_memory_limit: None,
            in_memory_persistence: false,
            key: None,
            location: None,
            loose: false,
            oauth: None,
            pwd: None,
            silent: false,
            skip_api_version_check: false,
            pid: None,
            child: None,
            _state: PhantomData,
        }
    }

    pub fn blob_host(mut self, v: impl Into<String>) -> Self {
        self.blob_host = Some(v.into());
        self
    }
    pub fn blob_keep_alive_timeout(mut self, v: u64) -> Self {
        self.blob_keep_alive_timeout = Some(v);
        self
    }
    pub fn blob_port(mut self, v: u16) -> Self {
        self.blob_port = Some(v);
        self
    }
    pub fn cert(mut self, v: impl Into<String>) -> Self {
        self.cert = Some(v.into());
        self
    }
    pub fn debug(mut self, path: impl Into<String>) -> Self {
        self.debug = Some(path.into());
        self
    }
    pub fn disable_product_style_url(mut self) -> Self {
        self.disable_product_style_url = true;
        self
    }
    pub fn disable_telemetry(mut self) -> Self {
        self.disable_telemetry = true;
        self
    }
    pub fn extent_memory_limit(mut self, mb: i64) -> Self {
        self.extent_memory_limit = Some(mb);
        self
    }
    pub fn in_memory_persistence(mut self) -> Self {
        self.in_memory_persistence = true;
        self
    }
    pub fn key(mut self, v: impl Into<String>) -> Self {
        self.key = Some(v.into());
        self
    }
    pub fn location(mut self, v: impl Into<String>) -> Self {
        self.location = Some(v.into());
        self
    }
    pub fn loose(mut self) -> Self {
        self.loose = true;
        self
    }
    pub fn oauth(mut self, level: impl Into<String>) -> Self {
        self.oauth = Some(level.into());
        self
    }
    pub fn pwd(mut self, v: impl Into<String>) -> Self {
        self.pwd = Some(v.into());
        self
    }
    pub fn silent(mut self) -> Self {
        self.silent = true;
        self
    }
    pub fn skip_api_version_check(mut self) -> Self {
        self.skip_api_version_check = true;
        self
    }

    /// Spawn the `azurite-blob` process with the configured options.
    pub fn start(mut self) -> io::Result<AzuriteBlob<()>> {
        let mut cmd = Command::new("azurite-blob");
        if let Some(ref v) = self.blob_host {
            cmd.args(["--blobHost", v]);
        }
        if let Some(v) = self.blob_keep_alive_timeout {
            cmd.args(["--blobKeepAliveTimeout", &v.to_string()]);
        }
        if let Some(v) = self.blob_port {
            cmd.args(["--blobPort", &v.to_string()]);
        }
        if let Some(ref v) = self.cert {
            cmd.args(["--cert", v]);
        }
        if let Some(ref v) = self.debug {
            cmd.args(["--debug", v]);
        }
        if self.disable_product_style_url {
            cmd.arg("--disableProductStyleUrl");
        }
        if self.disable_telemetry {
            cmd.arg("--disableTelemetry");
        }
        if let Some(v) = self.extent_memory_limit {
            cmd.args(["--extentMemoryLimit", &v.to_string()]);
        }
        if self.in_memory_persistence {
            cmd.arg("--inMemoryPersistence");
        }
        if let Some(ref v) = self.key {
            cmd.args(["--key", v]);
        }
        if let Some(ref v) = self.location {
            cmd.args(["--location", v]);
        }
        if self.loose {
            cmd.arg("--loose");
        }
        if let Some(ref v) = self.oauth {
            cmd.args(["--oauth", v]);
        }
        if let Some(ref v) = self.pwd {
            cmd.args(["--pwd", v]);
        }
        if self.silent {
            cmd.arg("--silent");
        }
        if self.skip_api_version_check {
            cmd.arg("--skipApiVersionCheck");
        }
        let child = cmd.spawn()?;
        Ok(AzuriteBlob {
            blob_host: self.blob_host.take(),
            blob_keep_alive_timeout: self.blob_keep_alive_timeout,
            blob_port: self.blob_port,
            cert: self.cert.take(),
            debug: self.debug.take(),
            disable_product_style_url: self.disable_product_style_url,
            disable_telemetry: self.disable_telemetry,
            extent_memory_limit: self.extent_memory_limit,
            in_memory_persistence: self.in_memory_persistence,
            key: self.key.take(),
            location: self.location.take(),
            loose: self.loose,
            oauth: self.oauth.take(),
            pwd: self.pwd.take(),
            silent: self.silent,
            skip_api_version_check: self.skip_api_version_check,
            pid: Some(child.id()),
            child: Some(child),
            _state: PhantomData,
        })
    }
}

impl AzuriteBlob<()> {
    /// Returns the PID of the spawned process.
    pub fn pid(&self) -> u32 {
        self.pid.expect("pid is set after start")
    }
}

impl Default for AzuriteBlob<Builder> {
    fn default() -> Self {
        Self::new()
    }
}

impl<State> Drop for AzuriteBlob<State> {
    fn drop(&mut self) {
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

/// Runs only the Azurite table storage service.
///
/// The type parameter `State` tracks whether the process has been started:
/// - [`AzuriteTable<Builder>`] — builder methods available, process not yet running.
/// - [`AzuriteTable<()>`] — process is running, `pid()` and similar methods available.
///
/// # Example
///
/// ```no_run
/// # use azurite_node_bindings::AzuriteTable;
/// let azurite_table = AzuriteTable::new()
///     .table_port(10002)
///     .in_memory_persistence()
///     .start()
///     .unwrap();
/// ```
pub struct AzuriteTable<State = Builder> {
    table_host: Option<String>,
    table_keep_alive_timeout: Option<u64>,
    table_port: Option<u16>,
    cert: Option<String>,
    debug: Option<String>,
    disable_product_style_url: bool,
    disable_telemetry: bool,
    in_memory_persistence: bool,
    key: Option<String>,
    location: Option<String>,
    loose: bool,
    oauth: Option<String>,
    pwd: Option<String>,
    silent: bool,
    skip_api_version_check: bool,
    pid: Option<u32>,
    child: Option<Child>,
    _state: PhantomData<State>,
}

impl AzuriteTable<Builder> {
    pub fn new() -> Self {
        Self {
            table_host: None,
            table_keep_alive_timeout: None,
            table_port: None,
            cert: None,
            debug: None,
            disable_product_style_url: false,
            disable_telemetry: false,
            in_memory_persistence: false,
            key: None,
            location: None,
            loose: false,
            oauth: None,
            pwd: None,
            silent: false,
            skip_api_version_check: false,
            pid: None,
            child: None,
            _state: PhantomData,
        }
    }

    pub fn table_host(mut self, v: impl Into<String>) -> Self {
        self.table_host = Some(v.into());
        self
    }
    pub fn table_keep_alive_timeout(mut self, v: u64) -> Self {
        self.table_keep_alive_timeout = Some(v);
        self
    }
    pub fn table_port(mut self, v: u16) -> Self {
        self.table_port = Some(v);
        self
    }
    pub fn cert(mut self, v: impl Into<String>) -> Self {
        self.cert = Some(v.into());
        self
    }
    pub fn debug(mut self, path: impl Into<String>) -> Self {
        self.debug = Some(path.into());
        self
    }
    pub fn disable_product_style_url(mut self) -> Self {
        self.disable_product_style_url = true;
        self
    }
    pub fn disable_telemetry(mut self) -> Self {
        self.disable_telemetry = true;
        self
    }
    pub fn in_memory_persistence(mut self) -> Self {
        self.in_memory_persistence = true;
        self
    }
    pub fn key(mut self, v: impl Into<String>) -> Self {
        self.key = Some(v.into());
        self
    }
    pub fn location(mut self, v: impl Into<String>) -> Self {
        self.location = Some(v.into());
        self
    }
    pub fn loose(mut self) -> Self {
        self.loose = true;
        self
    }
    pub fn oauth(mut self, level: impl Into<String>) -> Self {
        self.oauth = Some(level.into());
        self
    }
    pub fn pwd(mut self, v: impl Into<String>) -> Self {
        self.pwd = Some(v.into());
        self
    }
    pub fn silent(mut self) -> Self {
        self.silent = true;
        self
    }
    pub fn skip_api_version_check(mut self) -> Self {
        self.skip_api_version_check = true;
        self
    }

    /// Spawn the `azurite-table` process with the configured options.
    pub fn start(mut self) -> io::Result<AzuriteTable<()>> {
        let mut cmd = Command::new("azurite-table");
        if let Some(ref v) = self.table_host {
            cmd.args(["--tableHost", v]);
        }
        if let Some(v) = self.table_keep_alive_timeout {
            cmd.args(["--tableKeepAliveTimeout", &v.to_string()]);
        }
        if let Some(v) = self.table_port {
            cmd.args(["--tablePort", &v.to_string()]);
        }
        if let Some(ref v) = self.cert {
            cmd.args(["--cert", v]);
        }
        if let Some(ref v) = self.debug {
            cmd.args(["--debug", v]);
        }
        if self.disable_product_style_url {
            cmd.arg("--disableProductStyleUrl");
        }
        if self.disable_telemetry {
            cmd.arg("--disableTelemetry");
        }
        if self.in_memory_persistence {
            cmd.arg("--inMemoryPersistence");
        }
        if let Some(ref v) = self.key {
            cmd.args(["--key", v]);
        }
        if let Some(ref v) = self.location {
            cmd.args(["--location", v]);
        }
        if self.loose {
            cmd.arg("--loose");
        }
        if let Some(ref v) = self.oauth {
            cmd.args(["--oauth", v]);
        }
        if let Some(ref v) = self.pwd {
            cmd.args(["--pwd", v]);
        }
        if self.silent {
            cmd.arg("--silent");
        }
        if self.skip_api_version_check {
            cmd.arg("--skipApiVersionCheck");
        }
        let child = cmd.spawn()?;
        Ok(AzuriteTable {
            table_host: self.table_host.take(),
            table_keep_alive_timeout: self.table_keep_alive_timeout,
            table_port: self.table_port,
            cert: self.cert.take(),
            debug: self.debug.take(),
            disable_product_style_url: self.disable_product_style_url,
            disable_telemetry: self.disable_telemetry,
            in_memory_persistence: self.in_memory_persistence,
            key: self.key.take(),
            location: self.location.take(),
            loose: self.loose,
            oauth: self.oauth.take(),
            pwd: self.pwd.take(),
            silent: self.silent,
            skip_api_version_check: self.skip_api_version_check,
            pid: Some(child.id()),
            child: Some(child),
            _state: PhantomData,
        })
    }
}

impl AzuriteTable<()> {
    /// Returns the PID of the spawned process.
    pub fn pid(&self) -> u32 {
        self.pid.expect("pid is set after start")
    }
}

impl Default for AzuriteTable<Builder> {
    fn default() -> Self {
        Self::new()
    }
}

impl<State> Drop for AzuriteTable<State> {
    fn drop(&mut self) {
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

/// Runs only the Azurite queue storage service.
///
/// The type parameter `State` tracks whether the process has been started:
/// - [`AzuriteQueue<Builder>`] — builder methods available, process not yet running.
/// - [`AzuriteQueue<()>`] — process is running, `pid()` and similar methods available.
///
/// # Example
///
/// ```no_run
/// # use azurite_node_bindings::AzuriteQueue;
/// let azurite_queue = AzuriteQueue::new()
///     .queue_port(10001)
///     .in_memory_persistence()
///     .start()
///     .unwrap();
/// ```
pub struct AzuriteQueue<State = Builder> {
    queue_host: Option<String>,
    queue_port: Option<u16>,
    cert: Option<String>,
    debug: Option<String>,
    disable_product_style_url: bool,
    disable_telemetry: bool,
    extent_memory_limit: Option<i64>,
    in_memory_persistence: bool,
    key: Option<String>,
    location: Option<String>,
    loose: bool,
    oauth: Option<String>,
    pwd: Option<String>,
    silent: bool,
    skip_api_version_check: bool,
    pid: Option<u32>,
    child: Option<Child>,
    _state: PhantomData<State>,
}

impl AzuriteQueue<Builder> {
    pub fn new() -> Self {
        Self {
            queue_host: None,
            queue_port: None,
            cert: None,
            debug: None,
            disable_product_style_url: false,
            disable_telemetry: false,
            extent_memory_limit: None,
            in_memory_persistence: false,
            key: None,
            location: None,
            loose: false,
            oauth: None,
            pwd: None,
            silent: false,
            skip_api_version_check: false,
            pid: None,
            child: None,
            _state: PhantomData,
        }
    }

    pub fn queue_host(mut self, v: impl Into<String>) -> Self {
        self.queue_host = Some(v.into());
        self
    }
    pub fn queue_port(mut self, v: u16) -> Self {
        self.queue_port = Some(v);
        self
    }
    pub fn cert(mut self, v: impl Into<String>) -> Self {
        self.cert = Some(v.into());
        self
    }
    pub fn debug(mut self, path: impl Into<String>) -> Self {
        self.debug = Some(path.into());
        self
    }
    pub fn disable_product_style_url(mut self) -> Self {
        self.disable_product_style_url = true;
        self
    }
    pub fn disable_telemetry(mut self) -> Self {
        self.disable_telemetry = true;
        self
    }
    pub fn extent_memory_limit(mut self, mb: i64) -> Self {
        self.extent_memory_limit = Some(mb);
        self
    }
    pub fn in_memory_persistence(mut self) -> Self {
        self.in_memory_persistence = true;
        self
    }
    pub fn key(mut self, v: impl Into<String>) -> Self {
        self.key = Some(v.into());
        self
    }
    pub fn location(mut self, v: impl Into<String>) -> Self {
        self.location = Some(v.into());
        self
    }
    pub fn loose(mut self) -> Self {
        self.loose = true;
        self
    }
    pub fn oauth(mut self, level: impl Into<String>) -> Self {
        self.oauth = Some(level.into());
        self
    }
    pub fn pwd(mut self, v: impl Into<String>) -> Self {
        self.pwd = Some(v.into());
        self
    }
    pub fn silent(mut self) -> Self {
        self.silent = true;
        self
    }
    pub fn skip_api_version_check(mut self) -> Self {
        self.skip_api_version_check = true;
        self
    }

    /// Spawn the `azurite-queue` process with the configured options.
    pub fn start(mut self) -> io::Result<AzuriteQueue<()>> {
        let mut cmd = Command::new("azurite-queue");
        if let Some(ref v) = self.queue_host {
            cmd.args(["--queueHost", v]);
        }
        if let Some(v) = self.queue_port {
            cmd.args(["--queuePort", &v.to_string()]);
        }
        if let Some(ref v) = self.cert {
            cmd.args(["--cert", v]);
        }
        if let Some(ref v) = self.debug {
            cmd.args(["--debug", v]);
        }
        if self.disable_product_style_url {
            cmd.arg("--disableProductStyleUrl");
        }
        if self.disable_telemetry {
            cmd.arg("--disableTelemetry");
        }
        if let Some(v) = self.extent_memory_limit {
            cmd.args(["--extentMemoryLimit", &v.to_string()]);
        }
        if self.in_memory_persistence {
            cmd.arg("--inMemoryPersistence");
        }
        if let Some(ref v) = self.key {
            cmd.args(["--key", v]);
        }
        if let Some(ref v) = self.location {
            cmd.args(["--location", v]);
        }
        if self.loose {
            cmd.arg("--loose");
        }
        if let Some(ref v) = self.oauth {
            cmd.args(["--oauth", v]);
        }
        if let Some(ref v) = self.pwd {
            cmd.args(["--pwd", v]);
        }
        if self.silent {
            cmd.arg("--silent");
        }
        if self.skip_api_version_check {
            cmd.arg("--skipApiVersionCheck");
        }
        let child = cmd.spawn()?;
        Ok(AzuriteQueue {
            queue_host: self.queue_host.take(),
            queue_port: self.queue_port,
            cert: self.cert.take(),
            debug: self.debug.take(),
            disable_product_style_url: self.disable_product_style_url,
            disable_telemetry: self.disable_telemetry,
            extent_memory_limit: self.extent_memory_limit,
            in_memory_persistence: self.in_memory_persistence,
            key: self.key.take(),
            location: self.location.take(),
            loose: self.loose,
            oauth: self.oauth.take(),
            pwd: self.pwd.take(),
            silent: self.silent,
            skip_api_version_check: self.skip_api_version_check,
            pid: Some(child.id()),
            child: Some(child),
            _state: PhantomData,
        })
    }
}

impl AzuriteQueue<()> {
    /// Returns the PID of the spawned process.
    pub fn pid(&self) -> u32 {
        self.pid.expect("pid is set after start")
    }
}

impl Default for AzuriteQueue<Builder> {
    fn default() -> Self {
        Self::new()
    }
}

impl<State> Drop for AzuriteQueue<State> {
    fn drop(&mut self) {
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}
