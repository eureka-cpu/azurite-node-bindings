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
