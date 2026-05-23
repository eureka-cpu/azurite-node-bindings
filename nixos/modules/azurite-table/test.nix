{ pkgs, self, lib }:
pkgs.testers.runNixOSTest {
  name = "nixos-azurite-table";

  nodes.machine = { pkgs, ... }: {
    imports = [ self.nixosModules.azurite-table ];

    environment.systemPackages = [ pkgs.curl ];

    services.azurite-table = {
      enable = true;

      # Custom port/host to verify they are wired through
      tablePort = 14002;
      tableHost = "127.0.0.1";
      tableKeepAliveTimeout = 10;

      # Storage: no extentMemoryLimit on the table module
      inMemoryPersistence = true;

      # Behaviour flags
      loose = true;
      silent = true;
      skipApiVersionCheck = true;
      disableTelemetry = true;
      disableProductStyleUrl = true;
    };
  };

  testScript = ''
    machine.wait_for_unit("azurite-table.service")
    machine.succeed("systemctl is-active azurite-table.service")
    machine.wait_for_open_port(14002)

    # Verify every configured flag appears in the ExecStart command
    unit = machine.succeed("systemctl cat azurite-table.service")
    for flag in [
        "--tablePort 14002",
        "--tableHost 127.0.0.1",
        "--tableKeepAliveTimeout 10",
        "--inMemoryPersistence",
        "--loose",
        "--silent",
        "--skipApiVersionCheck",
        "--disableTelemetry",
        "--disableProductStyleUrl",
    ]:
        assert flag in unit, f"expected flag {flag!r} in ExecStart, got:\n{unit}"

    # Verify the table service responds over HTTP
    code = machine.succeed(
        "curl -s -o /dev/null -w '%{http_code}' http://127.0.0.1:14002/"
    ).strip()
    assert int(code) > 0, f"expected an HTTP response on port 14002, got code {code!r}"
  '';
}
