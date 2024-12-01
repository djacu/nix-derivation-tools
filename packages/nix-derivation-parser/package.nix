{ lib, rustPlatform }:
let
  inherit (lib)
    maintainers
    ;

  inherit (lib.fileset)
    toSource
    unions
    ;
in
rustPlatform.buildRustPackage {
  pname = "nix-derivation-parser";
  version = "0.1.0";

  src = toSource {
    root = ./.;
    fileset = unions [
      ./Cargo.toml
      ./Cargo.lock
      ./src
    ];
  };

  cargoLock.lockFile = ./Cargo.lock;

  meta = {
    description = "A Nix derivation parser and render written in Rust.";
    homepage = "https://github.com/djacu/nix-derivation-parser";
    license = lib.licenses.lgpl21;
    maintainers = with maintainers; [ djacu ];
  };
}
