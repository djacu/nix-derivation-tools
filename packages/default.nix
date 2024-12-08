inputs:
let
  inherit (inputs.nixpkgs)
    lib
    ;

  inherit (lib.attrsets)
    genAttrs
    ;

in
genAttrs
  [
    "x86_64-linux"
    "aarch64-linux"
  ]
  (system: {
    inherit (inputs.self.legacyPackages.${system})
      nix-derivation-parser
      ;
  })
