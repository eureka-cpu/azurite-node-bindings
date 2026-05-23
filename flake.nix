{
  description = ''
    Rust bindings for the Azurite Blob, Table and Queue storage node(s).

    This includes NixOS modules for development with NixOS tests.
  '';

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    treefmt = {
      url = "github:numtide/treefmt-nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, ... }@inputs:
    let
      inherit (nixpkgs) lib;
      eachSystem = f: lib.genAttrs lib.systems.flakeExposed (system: f nixpkgs.legacyPackages.${system});
    in
    {
      checks = eachSystem (pkgs: {
        azure-node-bindings = pkgs.rustPlatform.buildRustPackage {
          name = "azure-node-bindings";
          src = lib.cleanSourceWith {
            filter = path: _type: !lib.hasSuffix ".nix" path;
            src = lib.cleanSource ./.;
          };
          cargoLock.lockFile = ./Cargo.lock;
          doCheck = true;
          nativeCheckInputs = with pkgs; [ cargo rustc clippy rustfmt ];
          checkPhase = ''
            cargo fmt --check
            cargo clippy -- -Dwarnings
            cargo doc
          '';
        };
      });
      nixosModules = {
        azurite = import ./nixos/modules/azurite;
        azurite-blob = import ./nixos/modules/azurite-blob;
        azurite-table = import ./nixos/modules/azurite-table;
        azurite-queue = import ./nixos/modules/azurite-queue;
      };
      devShells = eachSystem (pkgs: {
        default = pkgs.mkShell {
          buildInputs = with pkgs; [
            cargo
            rustc
            rust-analyzer
            clippy
            rustfmt
            azurite
          ];
        };
      });
      formatter = eachSystem (pkgs:
        let
          treefmt = inputs.treefmt.lib;
          formattingOptions = {
            projectRootFile = "flake.lock";
            programs = {
              nixpkgs-fmt.enable = true;
              rustfmt.enable = true;
              taplo.enable = true;
            };
          };
        in
        treefmt.mkWrapper pkgs formattingOptions);
    };
}
