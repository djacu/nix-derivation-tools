{
  clippy,
  lib,
  rustPlatform,
}:
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

  strictDeps = true;

  nativeCheckInputs = [
    clippy
  ];

  preCheck = ''
    echo "Running clippy ..."
    cargo clippy --all -- \
      --warn clippy::all \
      --deny warnings
      # --warn clippy::pedantic \ TODO(@djacu): get all pedantic warnings fixed
      # --warn clippy::restriction \ TODO(@djacu): get all restriction warnings fixed
  '';

  meta = {
    description = "A Nix derivation parser and render written in Rust.";
    homepage = "https://github.com/djacu/nix-derivation-parser";
    license = lib.licenses.lgpl21;
    maintainers = with maintainers; [ djacu ];
  };
}
