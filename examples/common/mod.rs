use std::net::{SocketAddr, TcpStream};
use std::path::Path;
use std::time::{Duration, Instant};

/// Poll `host:port` until a TCP connection succeeds or `timeout` elapses.
pub fn wait_for_port(host: &str, port: u16, timeout: Duration) -> bool {
    let addr: SocketAddr = format!("{host}:{port}").parse().unwrap();
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        if TcpStream::connect_timeout(&addr, Duration::from_millis(100)).is_ok() {
            return true;
        }
        std::thread::sleep(Duration::from_millis(200));
    }
    false
}

/// Assert that a process with the given PID is no longer alive.
/// Checks `/proc/<pid>` on Linux.
pub fn assert_process_dead(pid: u32) {
    std::thread::sleep(Duration::from_millis(200));
    assert!(
        !Path::new(&format!("/proc/{pid}")).exists(),
        "process {pid} should have been killed on drop"
    );
}
