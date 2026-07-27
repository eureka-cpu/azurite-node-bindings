# Azurite Node Bindings

Rust and Nix bindings for the Azurite Blob, Table and Queue storage node(s).

## Usage

### As a Rust library

This is a wrapper around `std::process::Command` which uses the `azurite`
command line tool directly, unlike the [`testcontainer` crate flavor of `azurite`](https://docs.rs/testcontainers-modules/latest/testcontainers_modules/azurite/struct.Azurite.html)
which requires `docker`.

Each process is killed on `drop`.

#### Rust Example

The following example runs the toplevel `azurite` command and asserts all resource nodes
can be reached, and that their process is killed on `drop`.

> [!TIP]
> You can run this, as well as resource specific examples located in the `examples` folder:
>
> ```sh
> cargo run --example azurite
> ```

```rust,ignore
use azurite_node_bindings::Azurite;
use std::time::Duration;
use tracing::{info, warn};

mod common {
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
}
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
        // route azurite output through tracing
        .stdout(|line| info!(target: "azurite", "{line}"))
        .stderr(|line| warn!(target: "azurite", "{line}"))
        .start()
        .expect("failed to spawn azurite");

    let pid = azurite.pid();
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
```

### As a Nix library

The nix flake at the root of this repository exposes `nixosModules` which are intended to be used with [NixOS tests](https://nixos.org/manual/nixpkgs/unstable/#tester-runNixOSTest).

> [!NOTE]
> Each module's interface is derived from its command line counter-part, and can be found in the `nixos/modules` folder.
>
> If ever in doubt, all module options can be easily viewed from the `show-options` app:
>
> ```sh
> nix run github:eureka-cpu/azurite-node-bindings#show-options -- --help
> ```

For applications which rely on the Azure Blob, Table or Queue resource backends, this can be particularly
useful for testing purposes since you can get real isolation between a sandboxed Azurite node and other services
that will interact with it, more closely imitating production grade environments.

An `overlay` is also available, which packages `azurite`, updated on a nightly basis so you'll get the latest version
as soon as it's released.

#### Nix Example

The following example uploads a blob to an `azurite-blob` node behind an `nginx` reverse proxy using
the `azure-storage-blob` python library and asserts that it can be retrieved.

> [!TIP]
> You can run this example:
>
> ```sh
> nix build ./nixos/examples/azurite-blob#checks.<SYSTEM>.blob-upload-via-proxy -L
> ```

```nix
{
  description = "Example: upload a blob through an nginx proxy to an azurite-blob node";

  inputs = {
    azurite-node-bindings.url = "github:eureka-cpu/azurite-node-bindings";
    nixpkgs.follows = "azurite-node-bindings/nixpkgs";
  };

  outputs = { self, nixpkgs, azurite-node-bindings }:
    let
      eachSystem = f: nixpkgs.lib.genAttrs [
        "x86_64-linux"
        "aarch64-linux"
        "aarch64-darwin" # Requires nix-darwin linux-builder
      ]
        (system: f (nixpkgs.legacyPackages.${system}.extend azurite-node-bindings.overlays.default));
    in
    {
      checks = eachSystem (pkgs: {
        blob-upload-via-proxy = pkgs.testers.runNixOSTest {
          name = "blob-upload-via-proxy";

          nodes = {
            # Azurite blob storage: the backing store, not exposed to the client.
            provider = { config, ... }: {
              imports = [ azurite-node-bindings.nixosModules.azurite-blob ];
              services.azurite-blob = {
                enable = true;
                blobHost = "0.0.0.0";
                inMemoryPersistence = true;
                skipApiVersionCheck = true;
              };
              networking.firewall.allowedTCPPorts = [ config.services.azurite-blob.blobPort ];
            };

            # Nginx reverse proxy: the only endpoint the client talks to.
            server = { config, nodes, ... }: {
              services.nginx =
                let
                  blobPort = toString nodes.provider.services.azurite-blob.blobPort;
                in
                {
                  enable = true;
                  virtualHosts."default" = {
                    default = true;
                    locations."/" = {
                      proxyPass = "http://provider:${blobPort}";
                      extraConfig = ''
                        proxy_set_header Host "127.0.0.1:${blobPort}";
                      '';
                    };
                  };
                };
              networking.firewall.allowedTCPPorts = [ config.services.nginx.defaultHTTPListenPort ];
            };
          };

          # Include the azure python sdk
          extraPythonPackages = ps: [ ps.azure-storage-blob ];
          testScript = { nodes, ... }:
            let
              blobPort = toString nodes.provider.services.azurite-blob.blobPort;
              serverPort = toString nodes.server.services.nginx.defaultHTTPListenPort;
            in
            ''
              from azure.storage.blob import BlobServiceClient

              provider.start()
              provider.wait_for_unit("azurite-blob.service")
              provider.wait_for_open_port(${blobPort})

              server.start()
              server.wait_for_unit("nginx.service")
              server.wait_for_open_port(${serverPort})

              # Forward the server port to the host so we can reach it from the client
              server.forward_port(8080, ${serverPort})

              client = BlobServiceClient.from_connection_string(
                "DefaultEndpointsProtocol=http;"
                "AccountName=devstoreaccount1;"
                "AccountKey=Eby8vdM02xNOcqFlqUwJPLlmEtlCDXJ1OUzFT50uSRZ6IFsuFq2UVErCz4I6tq/K1SZFPTOtr/KBHBeksoGMGw==;"
                "BlobEndpoint=http://127.0.0.1:8080/devstoreaccount1;"
              )

              client.create_container("uploads")
              client.get_blob_client("uploads", "hello.txt").upload_blob(b"hello world")

              props = client.get_blob_client("uploads", "hello.txt").get_blob_properties()
              assert props["size"] == 11, "unexpected blob size: {}".format(props["size"])
            '';
        };
      });
    };
}
```

#### NixOS Test Resources

- The official NixOS test driver documentation: https://nixos.org/manual/nixos/stable/#sec-nixos-tests
- The official NixOS test developer guide: https://nix.dev/tutorials/nixos/integration-testing-using-virtual-machines
- In-depth guide by Applicative Systems: https://applicative.systems/nixos-test-driver-manual/
- Illustrative article by Brian McGee: https://bmcgee.ie/posts/2025/02/nixos-the-power-of-vm-tests/
- NixOS tests on macOS article: https://nixcademy.com/posts/running-nixos-integration-tests-on-macos/
