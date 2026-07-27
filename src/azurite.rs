use std::{
    io,
    marker::PhantomData,
    process::{Child, Command},
};

use crate::{Builder, IoSink};

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
    stdout: Option<IoSink>,
    stderr: Option<IoSink>,
    pid: Option<u32>,
    child: Option<Child>,
    _state: PhantomData<State>,
}

impl Azurite<Builder> {
    /// Create a new builder with default settings.
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
            stdout: None,
            stderr: None,
            pid: None,
            child: None,
            _state: PhantomData,
        }
    }

    /// Customize listening address for blob (defaults to `"127.0.0.1"`).
    pub fn blob_host(mut self, v: impl Into<String>) -> Self {
        self.blob_host = Some(v.into());
        self
    }
    /// Customize HTTP keep alive timeout for blob in seconds (defaults to `5`).
    pub fn blob_keep_alive_timeout(mut self, v: u64) -> Self {
        self.blob_keep_alive_timeout = Some(v);
        self
    }
    /// Customize listening port for blob (defaults to `10000`).
    pub fn blob_port(mut self, v: u16) -> Self {
        self.blob_port = Some(v);
        self
    }
    /// Customize listening address for queue (defaults to `"127.0.0.1"`).
    pub fn queue_host(mut self, v: impl Into<String>) -> Self {
        self.queue_host = Some(v.into());
        self
    }
    /// Customize HTTP keep alive timeout for queue in seconds (defaults to `5`).
    pub fn queue_keep_alive_timeout(mut self, v: u64) -> Self {
        self.queue_keep_alive_timeout = Some(v);
        self
    }
    /// Customize listening port for queue (defaults to `10001`).
    pub fn queue_port(mut self, v: u16) -> Self {
        self.queue_port = Some(v);
        self
    }
    /// Customize listening address for table (defaults to `"127.0.0.1"`).
    pub fn table_host(mut self, v: impl Into<String>) -> Self {
        self.table_host = Some(v.into());
        self
    }
    /// Customize HTTP keep alive timeout for table in seconds (defaults to `5`).
    pub fn table_keep_alive_timeout(mut self, v: u64) -> Self {
        self.table_keep_alive_timeout = Some(v);
        self
    }
    /// Customize listening port for table (defaults to `10002`).
    pub fn table_port(mut self, v: u16) -> Self {
        self.table_port = Some(v);
        self
    }
    /// Path to certificate file.
    pub fn cert(mut self, v: impl Into<String>) -> Self {
        self.cert = Some(v.into());
        self
    }
    /// Enable debug log by providing a valid local file path as log destination.
    pub fn debug(mut self, path: impl Into<String>) -> Self {
        self.debug = Some(path.into());
        self
    }
    /// Disable getting account name from the host of request URI, always get account name from
    /// the first path segment of request URI.
    pub fn disable_product_style_url(mut self) -> Self {
        self.disable_product_style_url = true;
        self
    }
    /// Disable telemetry data collection of this Azurite execution. By default, Azurite will
    /// collect telemetry data to help improve the product.
    pub fn disable_telemetry(mut self) -> Self {
        self.disable_telemetry = true;
        self
    }
    /// The number of megabytes to limit in-memory extent storage to. Only used with
    /// [`in_memory_persistence`](Self::in_memory_persistence). Defaults to 50% of total memory.
    pub fn extent_memory_limit(mut self, mb: i64) -> Self {
        self.extent_memory_limit = Some(mb);
        self
    }
    /// Disable persisting any data to disk. If the Azurite process is terminated, all data is lost.
    pub fn in_memory_persistence(mut self) -> Self {
        self.in_memory_persistence = true;
        self
    }
    /// Path to certificate key .pem file.
    pub fn key(mut self, v: impl Into<String>) -> Self {
        self.key = Some(v.into());
        self
    }
    /// Use an existing folder as workspace path, default is current working directory.
    pub fn location(mut self, v: impl Into<String>) -> Self {
        self.location = Some(v.into());
        self
    }
    /// Enable loose mode which ignores unsupported headers and parameters.
    pub fn loose(mut self) -> Self {
        self.loose = true;
        self
    }
    /// OAuth level. Candidate values: `"basic"`.
    pub fn oauth(mut self, level: impl Into<String>) -> Self {
        self.oauth = Some(level.into());
        self
    }
    /// Password for .pfx file.
    pub fn pwd(mut self, v: impl Into<String>) -> Self {
        self.pwd = Some(v.into());
        self
    }
    /// Disable access log displayed in console.
    pub fn silent(mut self) -> Self {
        self.silent = true;
        self
    }
    /// Skip the request API version check, request with all API versions will be allowed.
    pub fn skip_api_version_check(mut self) -> Self {
        self.skip_api_version_check = true;
        self
    }
    /// Redirect the process's stdout to the given sink.
    ///
    /// Accepts a [`Stdio`](std::process::Stdio) value or any `Fn(String) + Send + Sync + 'static`
    /// closure. When unset, the child process inherits the parent's stdout.
    pub fn stdout(mut self, sink: impl Into<IoSink>) -> Self {
        self.stdout = Some(sink.into());
        self
    }
    /// Redirect the process's stderr to the given sink.
    ///
    /// Accepts a [`Stdio`](std::process::Stdio) value or any `Fn(String) + Send + Sync + 'static`
    /// closure. When unset, the child process inherits the parent's stderr.
    pub fn stderr(mut self, sink: impl Into<IoSink>) -> Self {
        self.stderr = Some(sink.into());
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
        if let Some(sink) = self.stdout.take() {
            self.stdout = sink.configure_stdout(&mut cmd);
        }
        if let Some(sink) = self.stderr.take() {
            self.stderr = sink.configure_stderr(&mut cmd);
        }
        let mut child = cmd.spawn()?;
        if let Some(sink) = self.stdout.take() {
            sink.spawn_stdout_reader(&mut child);
        }
        if let Some(sink) = self.stderr.take() {
            sink.spawn_stderr_reader(&mut child);
        }
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
            stdout: None,
            stderr: None,
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
