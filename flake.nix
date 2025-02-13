{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-compat.url = "https://flakehub.com/f/edolstra/flake-compat/1.tar.gz";

    naersk.url = "github:nix-community/naersk/master";
    rust-overlay.url = "github:oxalica/rust-overlay";

    utils.url = "github:numtide/flake-utils";
  };

  outputs = {
    nixpkgs,
    utils,
    naersk,
    rust-overlay,
    ...
  }:
    utils.lib.eachDefaultSystem (
      system: let
        overlays = [(import rust-overlay)];

        pkgs = import nixpkgs {
          inherit system overlays;
        };

        naersk-lib = pkgs.callPackage naersk {};

        buildDeps = with pkgs; [pkg-config openssl];
        runtimeDeps = with pkgs; [pkg-config openssl];
      in {
        packages = {
          default = naersk-lib.buildPackage {
            nativeBuildInputs = buildDeps;
            buildInputs = runtimeDeps;

            src = ./.;
          };
        };

        devShell = pkgs.mkShell rec {
          nativeBuildInputs = buildDeps;

          buildInputs = with pkgs;
            [
              (rust-bin.stable.latest.default.override {
                extensions = ["rust-src"];
              })
              rust-analyzer-unwrapped
            ]
            ++ runtimeDeps;

          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath (buildInputs ++ nativeBuildInputs);
        };
      }
    );
}
