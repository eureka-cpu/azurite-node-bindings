{ pkgs, self, lib }:
pkgs.testers.runNixOSTest {
  name = "nixos-azurite-queue";

  nodes.machine = { pkgs, ... }: {
    imports = [ self.nixosModules.azurite-queue ];

    environment.systemPackages = [ pkgs.curl ];

    services.azurite-queue = {
      enable = true;

      # Custom port/host to verify they are wired through.
      # NOTE: azurite-queue itself has no keepAliveTimeout option
      # though it does exist on the azurite service for some reason.
      queuePort = 13001;
      queueHost = "127.0.0.1";

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
    machine.wait_for_unit("azurite-queue.service")
    machine.succeed("systemctl is-active azurite-queue.service")
    machine.wait_for_open_port(13001)

    # Verify every configured flag appears in the ExecStart command
    unit = machine.succeed("systemctl cat azurite-queue.service")
    for flag in [
        "--queuePort 13001",
        "--queueHost 127.0.0.1",
        "--inMemoryPersistence",
        "--extentMemoryLimit 128",
        "--loose",
        "--silent",
        "--skipApiVersionCheck",
        "--disableTelemetry",
        "--disableProductStyleUrl",
    ]:
        assert flag in unit, f"expected flag {flag!r} in ExecStart, got:\n{unit}"

    # Verify the queue service responds over HTTP
    code = machine.succeed(
        "curl -s -o /dev/null -w '%{http_code}' http://127.0.0.1:13001/"
    ).strip()
    assert int(code) > 0, f"expected an HTTP response on port 13001, got code {code!r}"
  '';
}
