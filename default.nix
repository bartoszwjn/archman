{ pkgs ? import <nixpkgs> { } }:

let lib = pkgs.lib;

in pkgs.rustPlatform.buildRustPackage rec {
  pname = "archman";
  version = "0.0.0";

  src = lib.sources.cleanSourceWith {
    filter = path: type: !(type == "directory" && baseNameOf path == "target");
    src = lib.sources.cleanSource
      (lib.sources.sourceFilesBySuffices ./. [ ".rs" ".toml" ".lock" ]);
  };

  cargoLock = { lockFile = ./Cargo.lock; };
}
