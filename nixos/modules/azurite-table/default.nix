{ config, lib, pkgs, ... }:
let
  cfg = config.services.azurite-table;

  args = lib.escapeShellArgs (
    [ "--tableHost" cfg.tableHost "--tablePort" (toString cfg.tablePort) "--tableKeepAliveTimeout" (toString cfg.tableKeepAliveTimeout) ]
    ++ lib.optionals (cfg.cert != null) [ "--cert" cfg.cert ]
    ++ lib.optionals (cfg.key != null) [ "--key" cfg.key ]
    ++ lib.optionals (cfg.pwd != null) [ "--pwd" cfg.pwd ]
    ++ lib.optionals (cfg.oauth != null) [ "--oauth" cfg.oauth ]
    ++ lib.optionals (cfg.debug != null) [ "--debug" cfg.debug ]
    ++ lib.optionals (cfg.location != null) [ "--location" cfg.location ]
    ++ lib.optional cfg.loose "--loose"
    ++ lib.optional cfg.silent "--silent"
    ++ lib.optional cfg.inMemoryPersistence "--inMemoryPersistence"
    ++ lib.optional cfg.disableTelemetry "--disableTelemetry"
    ++ lib.optional cfg.disableProductStyleUrl "--disableProductStyleUrl"
    ++ lib.optional cfg.skipApiVersionCheck "--skipApiVersionCheck"
  );
in
{
  options.services.azurite-table = {
    enable = lib.mkEnableOption "Azurite table storage emulator";

    package = lib.mkPackageOption pkgs "azurite" { };

    tableHost = lib.mkOption {
      type = lib.types.str;
      default = "127.0.0.1";
      description = "Listening address for the table service.";
    };

    tablePort = lib.mkOption {
      type = lib.types.port;
      default = 10002;
      description = "Listening port for the table service.";
    };

    tableKeepAliveTimeout = lib.mkOption {
      type = lib.types.int;
      default = 5;
      description = "HTTP keep-alive timeout (seconds) for the table service.";
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
        directory (<literal>/var/lib/azurite-table</literal>).
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
    systemd.services.azurite-table = {
      description = "Azurite table storage emulator";
      wantedBy = [ "multi-user.target" ];
      after = [ "network.target" ];
      serviceConfig = {
        ExecStart = "${cfg.package}/bin/azurite-table ${args}";
        DynamicUser = true;
        StateDirectory = "azurite-table";
        WorkingDirectory = "/var/lib/azurite-table";
        Restart = "on-failure";
      };
    };
  };
}
