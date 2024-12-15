inputs:
inputs.nixpkgs.lib.genAttrs
  [
    "x86_64-linux"
    "aarch64-linux"
  ]
  (
    system:
    let
      pkgs = inputs.self.legacyPackages.${system};
    in
    {
      bootstrap = pkgs.mkShell {
        packages =
          with pkgs;
          [
            cargo
            rustc
          ]
          ++ inputs.self.checks.${system}.pre-commit-check.enabledPackages;

        inherit (inputs.self.checks.${system}.pre-commit-check) shellHook;
      };
    }
  )
