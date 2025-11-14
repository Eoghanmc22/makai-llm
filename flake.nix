# NOTE: Run `just nix-init` to update Cargo.nix
#
# See Also: https://github.com/bevyengine/bevy/blob/v0.14.2/docs/linux_dependencies.md#nix
{
  description = "An inside joke discord bot";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    rust-overlay.url = "github:oxalica/rust-overlay";
    crate2nix.url = "github:nix-community/crate2nix";
  };

  outputs =
    inputs@{
      self,
      nixpkgs,
      flake-parts,
      rust-overlay,
      crate2nix,
      ...
    }:
      flake-parts.lib.mkFlake { inherit inputs; } {
        systems = nixpkgs.lib.systems.flakeExposed;
        perSystem = {self', pkgs, system, ...}:
          let
            pkgs = import nixpkgs {
              inherit system;
              overlays = [ rust-overlay.overlays.default ];
            };

            # rustToolchain = (pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml).override {
            #   extensions = [ "rust-src" ];
            #   targets = [ "aarch64-unknown-linux-gnu" "armv7-unknown-linux-gnueabihf" ];
            # };

            crateOverrides = {
              openssl-sys = {
                nativeBuildInputs = with pkgs; [
                  pkg-config
                ];
                buildInputs = with pkgs; [
                  openssl
                ];
              };
            };

            buildRustCrateForPkgs =
              crate:
              pkgs.buildRustCrate.override {
                # rustc = rustToolchain;
                # cargo = rustToolchain;

                defaultCrateOverrides = pkgs.defaultCrateOverrides // (builtins.mapAttrs (name: value: attrs: value) crateOverrides);
              };

            generatedCargoNix = ./Cargo.nix;

            cargoNix = import generatedCargoNix {
              inherit pkgs buildRustCrateForPkgs;
            };
          in {
            packages = rec {
              makai = cargoNix.workspaceMembers.makai.build;
              default = makai;
            };
            devShells.default = pkgs.mkShell {
              buildInputs = with pkgs; [
                pkg-config
                openssl
                just
                cargo
              ];
            };
          };
        flake = {
          nixosModules.default = {config, lib, system, ...}: let
            cfg = config.services.makai;
          in {
            options = {
              services.makai = {
                enable = lib.mkEnableOption "Enable makai llm";
                openaiEndpoint = lib.mkOption {
                  type = lib.types.str;
                };
                model = lib.mkOption {
                  type = lib.types.str;
                };
                # to pass discord token
                envFile = lib.mkOption {
                  type = lib.types.str;
                };
              };
            };

            config = lib.mkIf cfg.enable {
              systemd.services.makai = {
                description = "An inside joke discord bot";
                wantedBy = [ "multi-user.target" ];
                after = [ "network.target" ];
                environment = {
                  LLM_API = cfg.openaiEndpoint;
                  LLM_MODEL = cfg.model;
                  LLM_PROMPT_FILE = ./prompt.txt;
                  LLM_WORDS_FILE = ./prompt.txt;
                };
                serviceConfig = {
                  User = "makai";
                  Group = "makai";
                  EnvironmentFile = cfg.envFile;

                  ExecStart = "${lib.getExe self.packages.${system}.makai}";
                  WorkingDirectory = "/var/lib/makai";
                  StateDirectory = "makai";
                  RuntimeDirectory = "makai";
                  RuntimeDirectoryMode = "0755";
                  PrivateTmp = true;
                  DynamicUser = true;
                  DevicePolicy = "closed";
                  LockPersonality = true;
                  MemoryDenyWriteExecute = true;
                  PrivateUsers = true;
                  ProtectHome = true;
                  ProtectHostname = true;
                  ProtectKernelLogs = true;
                  ProtectKernelModules = true;
                  ProtectKernelTunables = true;
                  ProtectControlGroups = true;
                  ProcSubset = "all";
                  RestrictNamespaces = true;
                  RestrictRealtime = true;
                  SystemCallArchitectures = "native";
                  UMask = "0077";
                };
              };
            };
          };
        };
    };
}

