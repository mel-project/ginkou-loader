{
  description = "Environment to package a tauri app";

  inputs.nixpkgs.url = "github:nixos/nixpkgs/nixos-21.05";
  inputs.flake-utils.url = "github:numtide/flake-utils";
  inputs.melwalletd-flake.url = "github:themeliolabs/melwalletd";
  inputs.mozilla = { url = "github:mozilla/nixpkgs-mozilla"; flake = false; };

  outputs =
    { self
    , nixpkgs
    , mozilla
    , flake-utils
    , melwalletd-flake
    , ...
    } @inputs:
    let
      rust_channel = "1.52.0";
      rust_sha256 = "sha256-fcaq7+4shIvAy0qMuC3nnYGd0ZikkR5ln/rAruHA6mM=";
      rustOverlay = final: prev:
        let rustChannel = prev.rustChannelOf {
          channel = rust_channel;
          sha256  = rust_sha256;
        };
        in
        { inherit rustChannel;
          rustc = rustChannel.rust;
          cargo = rustChannel.rust;
        };

    in flake-utils.lib.eachDefaultSystem
      #["x86_64-linux"]
      (system: let

        pkgs = import nixpkgs {
          inherit system;
          overlays = [
            (import "${mozilla}/rust-overlay.nix")
            rustOverlay
          ];
        };

        rustPlatform = let rustChannel = pkgs.rustChannelOf {
            channel = rust_channel;
            sha256  = rust_sha256;
          }; in
            pkgs.makeRustPlatform {
              cargo = rustChannel.rust;
              rustc = rustChannel.rust;
            };

        gtk-deps = with pkgs; [
              (rustChannel.rust.override { extensions = [ "rust-src" ]; })
              binutils
              zlib
              glib
              libappindicator-gtk3
              webkit
              gtk3
              gtksourceview
        ];

        ginkou-loader = pkgs.callPackage ./ginkou-loader.nix {
          buildInputs = gtk-deps;

          nativeBuildInputs = with pkgs; [
            pkg-config
            llvmPackages_12.llvm
            llvmPackages_12.libclang
            llvmPackages_12.libcxxClang
            clang
          ];
        };

        in rec {
          packages.ginkou-loader = ginkou-loader;

          # Produces ginkou binary and melwalletd linked binary
          defaultPackage = ginkou-loader;
        });
}
