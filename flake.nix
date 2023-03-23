{
  description = "Trying to declaratively configure Arch Linux";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    fenix = {
      url = "github:nix-community/fenix";
      inputs = {
        nixpkgs.follows = "nixpkgs";
        rust-analyzer-src.follows = "";
      };
    };
    crane = {
      url = "github:ipetkov/crane";
      inputs = {
        nixpkgs.follows = "nixpkgs";
        flake-utils.follows = "flake-utils";
      };
    };
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
    fenix,
    crane,
  }: let
    mkOutputs = pkgs: let
      toolchainComponents = ["rustc" "cargo" "rustfmt" "clippy"];
      rustToolchain = (import fenix {inherit pkgs;}).stable.withComponents toolchainComponents;
      craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;

      src = craneLib.cleanCargoSource ./.;
      cargoArtifacts = craneLib.buildDepsOnly {
        inherit src;
      };
    in {
      archman = craneLib.buildPackage {
        inherit src cargoArtifacts;
        postBuild = ''
          target/release/archman completions > target/release/_archman
        '';
        postInstall = ''
          install -D --mode=444 target/release/_archman $out/share/zsh/site-functions/_archman
        '';
      };
      archman-clippy = craneLib.cargoClippy {
        inherit src cargoArtifacts;
        cargoClippyExtraArgs = "--all-targets -- --deny warnings";
      };
      archman-fmt = craneLib.cargoFmt {
        inherit src;
      };

      nix-fmt-check = pkgs.runCommandLocal "nix-fmt" {} ''
        ${pkgs.alejandra}/bin/alejandra --check ${self} 2>/dev/null
        touch $out
      '';
    };
  in
    flake-utils.lib.eachDefaultSystem (system: let
      pkgs = nixpkgs.legacyPackages.${system};
      outputs = mkOutputs pkgs;
    in {
      packages = {
        inherit (outputs) archman;
        default = outputs.archman;
      };

      checks = {inherit (outputs) archman archman-clippy archman-fmt nix-fmt-check;};

      devShells.default = pkgs.mkShell {
        inputsFrom = builtins.attrValues self.checks.${system};
      };

      formatter = pkgs.writeShellScriptBin "format-nix" ''
        ${pkgs.alejandra}/bin/alejandra "$@" 2>/dev/null;
      '';
    })
    // {
      overlays.default = final: prev: {inherit (mkOutputs final) rtangle;};
    };
}
