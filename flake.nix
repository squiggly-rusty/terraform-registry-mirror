{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };
        manifest = (pkgs.lib.importTOML ./Cargo.toml).package;
      in with pkgs; {
        devShells.default = mkShell {
          buildInputs = [
            rust-bin.stable.latest.default
            pkgs.rust-analyzer
            pkgs.pkg-config
            pkgs.libressl
          ];
        };
        packages.default = rustPlatform.buildRustPackage {
          pname = manifest.name;
          version = manifest.version;

          src = nix-gitignore.gitignoreSource [ "flake.nix" "flake.lock" ] ./.;
          cargoLock.lockFile = ./Cargo.lock;
        };
      });
}
