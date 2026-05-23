{ pkgs, self, lib }:
pkgs.testers.runNixOSTest {
  name = "nixos-azurite-blob";

  nodes.machine = { pkgs, ... }: {
    imports = [ self.nixosModules.azurite-blob ];

    environment.systemPackages = [ pkgs.curl ];

    services.azurite-blob = {
      enable = true;

      # Custom port/host to verify they are wired through
      blobPort = 12000;
      blobHost = "127.0.0.1";
      blobKeepAliveTimeout = 10;

      # Storage
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
    machine.wait_for_unit("azurite-blob.service")
    machine.succeed("systemctl is-active azurite-blob.service")
    machine.wait_for_open_port(12000)

    # Verify every configured flag appears in the ExecStart command
    unit = machine.succeed("systemctl cat azurite-blob.service")
    for flag in [
        "--blobPort 12000",
        "--blobHost 127.0.0.1",
        "--blobKeepAliveTimeout 10",
        "--inMemoryPersistence",
        "--extentMemoryLimit 128",
        "--loose",
        "--silent",
        "--skipApiVersionCheck",
        "--disableTelemetry",
        "--disableProductStyleUrl",
    ]:
        assert flag in unit, f"expected flag {flag!r} in ExecStart, got:\n{unit}"

    # Verify the blob service responds over HTTP
    code = machine.succeed(
        "curl -s -o /dev/null -w '%{http_code}' http://127.0.0.1:12000/"
    ).strip()
    assert int(code) > 0, f"expected an HTTP response on port 12000, got code {code!r}"
  '';
}
