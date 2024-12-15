inputs:
inputs.nixpkgs.lib.genAttrs
  [
    "x86_64-linux"
    "aarch64-linux"
    "x86_64-darwin"
    "aarch64-darwin"
  ]
  (
    system:
    let
      pkgs = inputs.self.legacyPackages.${system};
    in
    (inputs.treefmt-nix.lib.evalModule pkgs (
      { ... }:
      {
        imports = [
          ./genemichaels.nix
        ];

        config = {
          enableDefaultExcludes = true;
          projectRootFile = "flake.nix";
          programs = {
            genemichaels.enable = true;
            mdformat.enable = true;
            mdsh.enable = true;
            nixfmt.enable = true;
            shellcheck.enable = true;
          };
          settings.global.excludes = [
            "LICENSE"
            ".git-blame-ignore-revs"

            # nix-derivation-parser
            "**/nix-derivation-parser/**/*.drv"
            "**/nix-derivation-parser/.gitignore"
            "**/nix-derivation-parser/Cargo.toml"
          ];
        };
      }
    ))
  )
