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
//!         .skip_api_version_check()
//!         .disable_telemetry()
//!         .disable_product_style_url()
//!         // debug log
//!         .debug(debug_log.to_str().unwrap())
//!         // route azurite output through tracing
//!         .stdout(|line| info!(target: "AzuriteStdout", "{line}"))
//!         .stderr(|line| warn!(target: "AzuriteStderr", "{line}"))
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

use std::{
    io::{BufRead, BufReader},
    process::{Child, Command, Stdio},
    sync::Arc,
};

/// Marker type for the builder state.
#[doc(hidden)]
pub struct Builder;

/// Output destination for the child process's stdout or stderr.
///
/// Construct from [`std::process::Stdio`] or any `Fn(String) + Send + Sync + 'static` closure.
/// [`Arc<F>`](std::sync::Arc) where `F: Fn(String) + Send + Sync + 'static` also works
/// because the standard library blanket-implements `Fn` for `Arc<F>`.
///
/// Pass to [`Azurite::stdout`] / [`Azurite::stderr`] and the equivalent methods on
/// [`AzuriteBlob`], [`AzuriteTable`], and [`AzuriteQueue`].
///
/// [`Azurite::stdout`]: crate::Azurite::stdout
/// [`Azurite::stderr`]: crate::Azurite::stderr
/// [`AzuriteBlob`]: crate::AzuriteBlob
/// [`AzuriteTable`]: crate::AzuriteTable
/// [`AzuriteQueue`]: crate::AzuriteQueue
#[doc(hidden)]
pub struct IoSink(IoSinkKind);

enum IoSinkKind {
    Stdio(Stdio),
    Fn(Arc<dyn Fn(String) + Send + Sync + 'static>),
}

impl From<Stdio> for IoSink {
    fn from(s: Stdio) -> Self {
        IoSink(IoSinkKind::Stdio(s))
    }
}

impl<F: Fn(String) + Send + Sync + 'static> From<F> for IoSink {
    fn from(f: F) -> Self {
        IoSink(IoSinkKind::Fn(Arc::new(f)))
    }
}

impl IoSink {
    pub(crate) fn configure_stdout(self, cmd: &mut Command) -> Option<Self> {
        match self.0 {
            IoSinkKind::Stdio(s) => {
                cmd.stdout(s);
                None
            }
            IoSinkKind::Fn(_) => {
                cmd.stdout(Stdio::piped());
                Some(self)
            }
        }
    }

    pub(crate) fn configure_stderr(self, cmd: &mut Command) -> Option<Self> {
        match self.0 {
            IoSinkKind::Stdio(s) => {
                cmd.stderr(s);
                None
            }
            IoSinkKind::Fn(_) => {
                cmd.stderr(Stdio::piped());
                Some(self)
            }
        }
    }

    pub(crate) fn spawn_stdout_reader(self, child: &mut Child) {
        if let IoSinkKind::Fn(f) = self.0 {
            let pipe = child.stdout.take().expect("stdout piped");
            std::thread::spawn(move || {
                for line in BufReader::new(pipe).lines().map_while(Result::ok) {
                    f(line);
                }
            });
        }
    }

    pub(crate) fn spawn_stderr_reader(self, child: &mut Child) {
        if let IoSinkKind::Fn(f) = self.0 {
            let pipe = child.stderr.take().expect("stderr piped");
            std::thread::spawn(move || {
                for line in BufReader::new(pipe).lines().map_while(Result::ok) {
                    f(line);
                }
            });
        }
    }
}

mod azurite;
mod blob;
mod queue;
mod table;

pub use azurite::Azurite;
pub use blob::AzuriteBlob;
pub use queue::AzuriteQueue;
pub use table::AzuriteTable;
