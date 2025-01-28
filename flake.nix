{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    crane.url = "github:ipetkov/crane";
  };

  outputs = { self, nixpkgs, crane, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        craneLib = crane.mkLib pkgs;

        inherit (pkgs) lib;

        src = lib.fileset.toSource {
          root = ./.;
          fileset = lib.fileset.unions [
            # Default files from crane (Rust and cargo files)
            (craneLib.fileset.commonCargoSources ./.)
            # Include all .sql files as well
            ./migrations
            # Include cached queries
            # ./.sqlx
            ./Cargo.toml
            ./Cargo.lock
          ];
        };

        commonArgs = {
          inherit src;
          strictDeps = true;
          # tests require live DB and SaaS blockfrost, but nix doesn't allow network access
          # so we ignore tests
          doCheck = false;
        };
        bind = craneLib.buildPackage (commonArgs // {
          pname = "bind";
          cargoArtifacts = craneLib.buildDepsOnly commonArgs;
        });
      in {
        checks = { inherit bind; };

        packages.default = bind;
        packages.dockerImage = pkgs.dockerTools.buildImage {
          name = "ghcr.io/bind-labs/bind";
          tag = "latest";
          copyToRoot = [ bind ];
          config = {
            Cmd = [ "${bind}/bin/bind" ];
            User = "1000:1000";
          };
        };

        devShells.default = craneLib.devShell {
          checks = self.checks.${system};
          shellHook = ''
            export PGUSER=bind
            export PGPASSWORD=bind
            export PGDATABASE=bind
            export PGHOST=localhost
          '';
          packages = with pkgs; [
            rust-analyzer
            sqlx-cli
            cargo-watch
            postgresql
          ];
        };
      });
}
