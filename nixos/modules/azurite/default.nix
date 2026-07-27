{ config, lib, pkgs, ... }:
let
  cfg = config.services.azurite;

  args = lib.escapeShellArgs (
    [ "--blobHost" cfg.blobHost "--blobPort" (toString cfg.blobPort) "--blobKeepAliveTimeout" (toString cfg.blobKeepAliveTimeout) ]
    ++ [ "--queueHost" cfg.queueHost "--queuePort" (toString cfg.queuePort) "--queueKeepAliveTimeout" (toString cfg.queueKeepAliveTimeout) ]
    ++ [ "--tableHost" cfg.tableHost "--tablePort" (toString cfg.tablePort) "--tableKeepAliveTimeout" (toString cfg.tableKeepAliveTimeout) ]
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
  options.services.azurite = {
    enable = lib.mkEnableOption "Azurite Azure Storage emulator (blob, table and queue)";

    package = lib.mkPackageOption pkgs "azurite" { };

    blobHost = lib.mkOption {
      type = lib.types.str;
      default = "127.0.0.1";
      description = "Customize listening address for blob (defaults to \"127.0.0.1\").";
    };

    blobPort = lib.mkOption {
      type = lib.types.port;
      default = 10000;
      description = "Customize listening port for blob (defaults to 10000).";
    };

    blobKeepAliveTimeout = lib.mkOption {
      type = lib.types.int;
      default = 5;
      description = "Customize HTTP keep alive timeout for blob in seconds (defaults to 5).";
    };

    queueHost = lib.mkOption {
      type = lib.types.str;
      default = "127.0.0.1";
      description = "Customize listening address for queue (defaults to \"127.0.0.1\").";
    };

    queuePort = lib.mkOption {
      type = lib.types.port;
      default = 10001;
      description = "Customize listening port for queue (defaults to 10001).";
    };

    queueKeepAliveTimeout = lib.mkOption {
      type = lib.types.int;
      default = 5;
      description = "Customize HTTP keep alive timeout for queue in seconds (defaults to 5).";
    };

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

    extentMemoryLimit = lib.mkOption {
      type = lib.types.nullOr lib.types.int;
      default = null;
      description = "The number of megabytes to limit in-memory extent storage to. Only used with the --inMemoryPersistence option. Defaults to 50% of total memory.";
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
    systemd.services.azurite = {
      description = "Azurite Azure Storage emulator (blob, table and queue)";
      wantedBy = [ "multi-user.target" ];
      after = [ "network.target" ];
      serviceConfig = {
        ExecStart = "${cfg.package}/bin/azurite ${args}";
        DynamicUser = true;
        StateDirectory = "azurite";
        WorkingDirectory = "/var/lib/azurite";
        Restart = "on-failure";
      };
    };
  };
}
