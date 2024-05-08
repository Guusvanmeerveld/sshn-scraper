{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = {
    # self,
    nixpkgs,
    crane,
    flake-utils,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (system: let
      pkgs = nixpkgs.legacyPackages.${system};

      inherit (pkgs) lib;

      craneLib = crane.lib.${system};

      fileSetForCrate = crate:
        lib.fileset.toSource {
          root = ./.;
          fileset = lib.fileset.unions [
            ./Cargo.toml
            ./Cargo.lock
            ./sshn-lib
            crate
          ];
        };

      sshn-cli = craneLib.buildPackage {
        inherit (craneLib.crateNameFromCargoToml {cargoToml = ./sshn-cli/Cargo.toml;}) version pname;

        nativeBuildInputs = with pkgs; [pkg-config openssl];

        buildInputs = with pkgs; [chromium chromedriver];

        cargoExtraArgs = "-p sshn-cli";

        src = fileSetForCrate ./sshn-cli;

        # Add extra inputs here or any other derivation settings
        doCheck = false;
        # buildInputs = [];
      };
    in {
      packages = {
        sshn-cli = sshn-cli;
        default = sshn-cli;
      };
    });
}
