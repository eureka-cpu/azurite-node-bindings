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
      description = "Customize listening address for table (defaults to \"127.0.0.1\").";
    };

    tablePort = lib.mkOption {
      type = lib.types.port;
      default = 10002;
      description = "Customize listening port for table (defaults to 10002).";
    };

    tableKeepAliveTimeout = lib.mkOption {
      type = lib.types.int;
      default = 5;
      description = "Customize HTTP keep alive timeout for table in seconds (defaults to 5).";
    };

    cert = lib.mkOption {
      type = lib.types.nullOr lib.types.path;
      default = null;
      description = "Path to certificate file.";
    };

    key = lib.mkOption {
      type = lib.types.nullOr lib.types.path;
      default = null;
      description = "Path to certificate key .pem file.";
    };

    pwd = lib.mkOption {
      type = lib.types.nullOr lib.types.str;
      default = null;
      description = "Password for .pfx file.";
    };

    oauth = lib.mkOption {
      type = lib.types.nullOr (lib.types.enum [ "basic" ]);
      default = null;
      description = "OAuth level. Candidate values: \"basic\".";
    };

    debug = lib.mkOption {
      type = lib.types.nullOr lib.types.path;
      default = null;
      description = "Enable debug log by providing a valid local file path as log destination.";
    };

    location = lib.mkOption {
      type = lib.types.nullOr lib.types.path;
      default = null;
      description = "Use an existing folder as workspace path, default is current working directory.";
    };

    loose = lib.mkOption {
      type = lib.types.bool;
      default = false;
      description = "Enable loose mode which ignores unsupported headers and parameters.";
    };

    silent = lib.mkOption {
      type = lib.types.bool;
      default = false;
      description = "Disable access log displayed in console.";
    };

    inMemoryPersistence = lib.mkOption {
      type = lib.types.bool;
      default = false;
      description = "Disable persisting any data to disk. If the Azurite process is terminated, all data is lost.";
    };

    disableTelemetry = lib.mkOption {
      type = lib.types.bool;
      default = false;
      description = "Disable telemetry data collection of this Azurite execution. By default, Azurite will collect telemetry data to help improve the product.";
    };

    disableProductStyleUrl = lib.mkOption {
      type = lib.types.bool;
      default = false;
      description = "Disable getting account name from the host of request URI, always get account name from the first path segment of request URI.";
    };

    skipApiVersionCheck = lib.mkOption {
      type = lib.types.bool;
      default = false;
      description = "Skip the request API version check, request with all API versions will be allowed.";
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
