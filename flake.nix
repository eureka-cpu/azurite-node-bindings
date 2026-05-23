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

      src = lib.cleanSourceWith {
        filter = path: _type: !lib.hasSuffix ".nix" path;
        src = lib.cleanSource ./.;
      };

      formattingOptions = {
        projectRootFile = "flake.lock";
        programs = {
          nixpkgs-fmt.enable = true;
          rustfmt.enable = true;
          taplo.enable = true;
          yamlfmt.enable = true;
          mdformat.enable = true;
        };
      };
    in
    {
      checks = eachSystem (pkgs: {
        azure-node-bindings = pkgs.rustPlatform.buildRustPackage {
          inherit src;
          name = "azure-node-bindings";
          cargoLock.lockFile = ./Cargo.lock;
          doCheck = true;
          nativeCheckInputs = with pkgs; [ cargo rustc clippy rustfmt ];
          checkPhase = ''
            cargo fmt --check
            cargo clippy -- -Dwarnings
          '';
        };

        docs = pkgs.rustPlatform.buildRustPackage {
          inherit src;
          name = "azure-node-bindings-docs";
          cargoLock.lockFile = ./Cargo.lock;
          buildPhase = "cargo doc --no-deps";
          installPhase = ''
            mv target/doc $out
            echo '<meta http-equiv="refresh" content="0; url=azurite_node_bindings/index.html">' \
              > $out/index.html
          '';
          doCheck = false;
        };

        fmt =
          let
            treefmt =
              let
                treefmt = import inputs.treefmt;
              in
              treefmt.evalModule pkgs formattingOptions;
          in
          treefmt.config.build.check ./.;
      } // lib.optionalAttrs pkgs.stdenv.isLinux {
        nixos-azurite = import ./nixos/modules/azurite/test.nix { inherit pkgs self lib; };
        nixos-azurite-blob = import ./nixos/modules/azurite-blob/test.nix { inherit pkgs self lib; };
        nixos-azurite-queue = import ./nixos/modules/azurite-queue/test.nix { inherit pkgs self lib; };
        nixos-azurite-table = import ./nixos/modules/azurite-table/test.nix { inherit pkgs self lib; };
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
        in
        treefmt.mkWrapper pkgs formattingOptions);
    };
}
