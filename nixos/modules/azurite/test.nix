{ pkgs, self, lib }:
pkgs.testers.runNixOSTest {
  name = "nixos-azurite";

  nodes.machine = { pkgs, ... }: {
    imports = [ self.nixosModules.azurite ];

    environment.systemPackages = [ pkgs.curl ];

    services.azurite = {
      enable = true;

      # Custom ports to verify port options are wired through
      blobPort = 11000;
      queuePort = 11001;
      tablePort = 11002;

      # Custom hosts
      blobHost = "127.0.0.1";
      queueHost = "127.0.0.1";
      tableHost = "127.0.0.1";

      # Keep-alive timeouts
      blobKeepAliveTimeout = 10;
      queueKeepAliveTimeout = 10;
      tableKeepAliveTimeout = 10;

      # Storage: keep everything in memory
      inMemoryPersistence = true;
      extentMemoryLimit = 128;

      # Behaviour flags
      loose = true;
      silent = true;
      skipApiVersionCheck = true;
      disableTelemetry = true;
      disableProductStyleUrl = true;
    };
  };

  testScript = ''
    machine.wait_for_unit("azurite.service")
    machine.succeed("systemctl is-active azurite.service")

    # All three ports must be open on the configured (non-default) values
    machine.wait_for_open_port(11000)
    machine.wait_for_open_port(11001)
    machine.wait_for_open_port(11002)

    # Verify every configured flag appears in the ExecStart command
    unit = machine.succeed("systemctl cat azurite.service")
    for flag in [
        "--blobPort 11000",
        "--queuePort 11001",
        "--tablePort 11002",
        "--blobHost 127.0.0.1",
        "--queueHost 127.0.0.1",
        "--tableHost 127.0.0.1",
        "--blobKeepAliveTimeout 10",
        "--queueKeepAliveTimeout 10",
        "--tableKeepAliveTimeout 10",
        "--inMemoryPersistence",
        "--extentMemoryLimit 128",
        "--loose",
        "--silent",
        "--skipApiVersionCheck",
        "--disableTelemetry",
        "--disableProductStyleUrl",
    ]:
        assert flag in unit, f"expected flag {flag!r} in ExecStart, got:\n{unit}"

    # All three services respond over HTTP (Azurite returns XML errors for
    # unauthenticated requests, which is still a valid HTTP response)
    for port in [11000, 11001, 11002]:
        machine.succeed(f"curl -sf --max-time 5 http://127.0.0.1:{port}/ || true")
        code = machine.succeed(
            f"curl -s -o /dev/null -w '%{{http_code}}' http://127.0.0.1:{port}/"
        ).strip()
        assert int(code) > 0, f"expected an HTTP response on port {port}, got code {code!r}"
  '';
}
