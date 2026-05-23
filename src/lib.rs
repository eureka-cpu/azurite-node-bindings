#![doc = include_str!("../README.md")]
//!
//! The toplevel `azurite` command can be ran using [`Azurite`], or each node
//! can be ran individually using [`AzuriteBlob`], [`AzuriteTable`] and [`AzuriteQueue`].

use std::io;
use std::process::{Child, Command};

/// Runs all three Azurite services (blob, table, queue) in a single process.
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
pub struct Azurite {
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
}

impl Azurite {
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

    /// Returns the PID of the spawned process, or `None` if not yet started.
    pub fn pid(&self) -> Option<u32> {
        self.pid
    }

    /// Spawn the `azurite` process with the configured options.
    pub fn start(mut self) -> io::Result<Self> {
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
        self.pid = Some(child.id());
        self.child = Some(child);
        Ok(self)
    }
}

impl Default for Azurite {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for Azurite {
    fn drop(&mut self) {
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

/// Runs only the Azurite blob storage service.
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
pub struct AzuriteBlob {
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
}

impl AzuriteBlob {
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

    /// Returns the PID of the spawned process, or `None` if not yet started.
    pub fn pid(&self) -> Option<u32> {
        self.pid
    }

    /// Spawn the `azurite-blob` process with the configured options.
    pub fn start(mut self) -> io::Result<Self> {
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
        self.pid = Some(child.id());
        self.child = Some(child);
        Ok(self)
    }
}

impl Default for AzuriteBlob {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for AzuriteBlob {
    fn drop(&mut self) {
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

/// Runs only the Azurite table storage service.
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
pub struct AzuriteTable {
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
}

impl AzuriteTable {
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

    /// Returns the PID of the spawned process, or `None` if not yet started.
    pub fn pid(&self) -> Option<u32> {
        self.pid
    }

    /// Spawn the `azurite-table` process with the configured options.
    pub fn start(mut self) -> io::Result<Self> {
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
        self.pid = Some(child.id());
        self.child = Some(child);
        Ok(self)
    }
}

impl Default for AzuriteTable {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for AzuriteTable {
    fn drop(&mut self) {
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

/// Runs only the Azurite queue storage service.
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
pub struct AzuriteQueue {
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
}

impl AzuriteQueue {
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

    /// Returns the PID of the spawned process, or `None` if not yet started.
    pub fn pid(&self) -> Option<u32> {
        self.pid
    }

    /// Spawn the `azurite-queue` process with the configured options.
    pub fn start(mut self) -> io::Result<Self> {
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
        self.pid = Some(child.id());
        self.child = Some(child);
        Ok(self)
    }
}

impl Default for AzuriteQueue {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for AzuriteQueue {
    fn drop(&mut self) {
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}
