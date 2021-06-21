{
  description = "Environment to package a tauri app";

  inputs.nixpkgs.url = "github:nixos/nixpkgs/nixos-21.05";
  inputs.flake-utils.url = "github:numtide/flake-utils";
  inputs.melwalletd-flake.url = "github:themeliolabs/melwalletd";
  inputs.mozilla = { url = "github:mozilla/nixpkgs-mozilla"; flake = false; };
  inputs.ginkou-src = { url = "github:themeliolabs/ginkou"; flake = false; };
  #inputs.ginkou-src = { url = "/home/casper/Programming/themelio/ginkou"; flake = false; };
  inputs.ginkou-loader-src = { url = "github:themeliolabs/ginkou-loader"; flake = false; };

  outputs =
    { self
    , nixpkgs
    , mozilla
    , flake-utils
    , melwalletd-flake
    , ginkou-loader-src
    , ginkou-src
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

    in flake-utils.lib.eachSystem
      ["x86_64-linux"]
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

        ginkou-loader = rustPlatform.buildRustPackage rec {
          pname = "ginkou-loader-v${version}";
          version = "0.1.0";
          src = "${ginkou-loader-src}";
          cargoSha256 = "sha256-foJmaD2ZjHekIEHo/O/NssoGoywLpOE5diQ8oykUKSQ=";
        };

        ginkou = pkgs.callPackage "${ginkou-src}/rollup-build.nix" {
          nodejs = pkgs.nodejs-12_x;
          rollup = pkgs.nodePackages.rollup;
        };

        melwalletd = melwalletd-flake.packages."${system}".melwalletd;

        bundle = pkgs.callPackage ./bundle.nix {
          inherit melwalletd ginkou ginkou-loader;
        };

        tauri-deps = with pkgs; [
              (rustChannel.rust.override { extensions = [ "rust-src" ]; })
              binutils
              zlib
              wget
              curl
              openssl
              squashfsTools
              pkg-config
              libsoup

              webkit
              gtk3-x11
              gtksourceview
              libayatana-appindicator-gtk3
        ];

        in rec {
          packages.ginkou-loader = ginkou-loader;

          # Produces ginkou binary and melwalletd linked binary
          defaultPackage = bundle;

          /*
          devShell = pkgs.mkShell {
            buildInputs = with pkgs; [
              docker
              #packages.tauri
            ] ++ tauri-deps;

            shellHook = ''
              export OPENSSL_DIR="${pkgs.openssl.dev}"
              export OPENSSL_LIB_DIR="${pkgs.openssl.out}/lib"

              # melwalletd
              export PATH=$PATH:${melwalletd}/bin

              # Copy in ginkou repo
              cp -r ${ginkou} ./ginkou

              # Make writable for building
              chmod +w ginkou
              chmod +w ginkou/public
              mkdir ginkou/public/build

              # Place into project with target triplet for bundling
              cp ${melwalletd}/bin/melwalletd ./src-tauri/melwalletd-$(gcc -dumpmachine)

              # Tauri cli
              export PATH=$PATH:${packages.tauri}/bin
              alias tauri='cargo-tauri tauri'
            '';
          };
          */
        });
}
