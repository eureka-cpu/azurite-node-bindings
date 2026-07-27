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

      overlays.default = import ./nixos/overlays;
      eachSystem = f: lib.genAttrs lib.systems.flakeExposed (system:
        let
          pkgs = nixpkgs.legacyPackages.${system}.extend overlays.default;
        in
        f pkgs);

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
      inherit overlays;

      legacyPackages = eachSystem lib.id;

      apps = eachSystem (pkgs: {
        update-azurite = {
          type = "app";
          program = "${pkgs.azurite.passthru.updateScript}";
          meta.description = "Internal update script for the azurite package.";
        };
        show-options = {
          type = "app";
          program = "${pkgs.callPackage ./nixos/modules/show-options { }}/bin/azurite-show-options";
          meta.description = "Show the NixOS module interface options declared by azurite-node-bindings.";
        };
      });

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
