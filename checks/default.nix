inputs:
let
  # inherits
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
    formatting = inputs.self.formatterModule.${system}.config.build.check inputs.self;

    pre-commit-check = inputs.pre-commit-hooks.lib.${system}.run {
      src = inputs.self;
      hooks = {
        treefmt.enable = true;
        treefmt.packageOverrides.treefmt = inputs.self.formatter.${system};
      };
    };
  })
