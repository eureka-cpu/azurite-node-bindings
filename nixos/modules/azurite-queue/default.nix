{ config, lib, pkgs, ... }:
let
  cfg = config.services.azurite-queue;

  args = lib.escapeShellArgs (
    [ "--queueHost" cfg.queueHost "--queuePort" (toString cfg.queuePort) ]
    ++ lib.optionals (cfg.cert != null) [ "--cert" cfg.cert ]
    ++ lib.optionals (cfg.key != null) [ "--key" cfg.key ]
    ++ lib.optionals (cfg.pwd != null) [ "--pwd" cfg.pwd ]
    ++ lib.optionals (cfg.oauth != null) [ "--oauth" cfg.oauth ]
    ++ lib.optionals (cfg.debug != null) [ "--debug" cfg.debug ]
    ++ lib.optionals (cfg.location != null) [ "--location" cfg.location ]
    ++ lib.optionals (cfg.extentMemoryLimit != null) [ "--extentMemoryLimit" (toString cfg.extentMemoryLimit) ]
    ++ lib.optional cfg.loose "--loose"
    ++ lib.optional cfg.silent "--silent"
    ++ lib.optional cfg.inMemoryPersistence "--inMemoryPersistence"
    ++ lib.optional cfg.disableTelemetry "--disableTelemetry"
    ++ lib.optional cfg.disableProductStyleUrl "--disableProductStyleUrl"
    ++ lib.optional cfg.skipApiVersionCheck "--skipApiVersionCheck"
  );
in
{
  options.services.azurite-queue = {
    enable = lib.mkEnableOption "Azurite queue storage emulator";

    package = lib.mkPackageOption pkgs "azurite" { };

    queueHost = lib.mkOption {
      type = lib.types.str;
      default = "127.0.0.1";
      description = "Listening address for the queue service.";
    };

    queuePort = lib.mkOption {
      type = lib.types.port;
      default = 10001;
      description = "Listening port for the queue service.";
    };

    cert = lib.mkOption {
      type = lib.types.nullOr lib.types.path;
      default = null;
      description = "Path to a TLS certificate file.";
    };

    key = lib.mkOption {
      type = lib.types.nullOr lib.types.path;
      default = null;
      description = "Path to the TLS certificate key (.pem) file.";
    };

    pwd = lib.mkOption {
      type = lib.types.nullOr lib.types.str;
      default = null;
      description = "Password for a .pfx certificate file.";
    };

    oauth = lib.mkOption {
      type = lib.types.nullOr (lib.types.enum [ "basic" ]);
      default = null;
      description = "OAuth authentication level. Set to \"basic\" to enable.";
    };

    debug = lib.mkOption {
      type = lib.types.nullOr lib.types.path;
      default = null;
      description = "Enable debug logging; value is the path to the log file.";
    };

    location = lib.mkOption {
      type = lib.types.nullOr lib.types.path;
      default = null;
      description = ''
        Workspace directory for persisted data. Defaults to the service state
        directory (<literal>/var/lib/azurite-queue</literal>).
      '';
    };

    extentMemoryLimit = lib.mkOption {
      type = lib.types.nullOr lib.types.int;
      default = null;
      description = ''
        Maximum in-memory extent storage in megabytes. Only used with
        <option>inMemoryPersistence</option>. Defaults to 50 % of total RAM.
      '';
    };

    loose = lib.mkOption {
      type = lib.types.bool;
      default = false;
      description = "Ignore unsupported headers and parameters (loose mode).";
    };

    silent = lib.mkOption {
      type = lib.types.bool;
      default = false;
      description = "Suppress the access log on stdout.";
    };

    inMemoryPersistence = lib.mkOption {
      type = lib.types.bool;
      default = false;
      description = "Keep all data in memory; nothing is written to disk.";
    };

    disableTelemetry = lib.mkOption {
      type = lib.types.bool;
      default = false;
      description = "Opt out of Azurite telemetry collection.";
    };

    disableProductStyleUrl = lib.mkOption {
      type = lib.types.bool;
      default = false;
      description = "Always derive the account name from the first URL path segment instead of the host.";
    };

    skipApiVersionCheck = lib.mkOption {
      type = lib.types.bool;
      default = false;
      description = "Accept requests regardless of their API version header.";
    };
  };

  config = lib.mkIf cfg.enable {
    systemd.services.azurite-queue = {
      description = "Azurite queue storage emulator";
      wantedBy = [ "multi-user.target" ];
      after = [ "network.target" ];
      serviceConfig = {
        ExecStart = "${cfg.package}/bin/azurite-queue ${args}";
        DynamicUser = true;
        StateDirectory = "azurite-queue";
        WorkingDirectory = "/var/lib/azurite-queue";
        Restart = "on-failure";
      };
    };
  };
}
