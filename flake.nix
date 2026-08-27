{
  description = "rust app flake";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs";
    flake-utils.url = "github:numtide/flake-utils";

    naersk = {
      url = "github:nix-community/naersk";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { nixpkgs, flake-utils, naersk, rust-overlay, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        name = "macro-lisp";

        pkgs = (import nixpkgs) {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };
        toolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;

        nativeBuildInputs = with pkgs; [
          pkg-config
        ];
        buildInputs = with pkgs; [];

        naersk' = pkgs.callPackage naersk {
          rustc = toolchain;
          cargo = toolchain;
        };
      in rec {
        packages = {
          default = packages.${name};

          ${name} = naersk'.buildPackage {
            inherit nativeBuildInputs;
            inherit buildInputs;

            src = ./.;
          };
        };

        devShells.default = pkgs.mkShell {
          inherit buildInputs;
          nativeBuildInputs = nativeBuildInputs ++ [ toolchain ];
        };
      }
    );
}
