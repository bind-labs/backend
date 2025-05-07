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
            ./.sqlx
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
        api = craneLib.buildPackage (commonArgs // {
          pname = "api";
          cargoArtifacts = craneLib.buildDepsOnly commonArgs;
        });
      in {
        checks = { api = api; };

        packages.default = api;
        packages.dockerImage = pkgs.dockerTools.buildImage {
          name = "ghcr.io/bind-labs/api";
          tag = "latest";
          copyToRoot = [ api ];
          config = {
            Cmd = [ "${api}/bin/api" ];
            User = "1000:1000";
          };
        };

        devShells.default = let
          start-postgres = pkgs.writeShellScriptBin "start-postgres" ''
            #!${pkgs.bash}/bin/bash
            echo "Starting postgres..."
            docker run -p 5432:5432 -e POSTGRES_PASSWORD=bind -e POSTGRES_USER=bind -e POSTGRES_DB=bind --name bind-postgres -d postgres >/dev/null \
              || docker start bind-postgres >/dev/null
            echo "Running migrations..."
            sleep 1 && sqlx migrate run
          '';
          recreate-postgres = pkgs.writeShellScriptBin "recreate-postgres" ''
            #!${pkgs.bash}/bin/bash
            destroy-postgres
            start-postgres
          '';
          destroy-postgres = pkgs.writeShellScriptBin "destroy-postgres" ''
            #!${pkgs.bash}/bin/bash
            echo "Destroying postgres..."
            docker stop bind-postgres >/dev/null
            docker rm bind-postgres >/dev/null
          '';
        in craneLib.devShell {
          checks = self.checks.${system};
          shellHook = ''
            export PGUSER=bind
            export PGPASSWORD=bind
            export PGDATABASE=bind
            export PGHOST=localhost
          '';
          packages = with pkgs; [
            sqlx-cli
            cargo-watch
            postgresql
            start-postgres
            recreate-postgres
            destroy-postgres
          ];
        };
      });
}
