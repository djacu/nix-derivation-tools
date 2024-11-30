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
      bootstrap = pkgs.mkShellNoCC {
        packages = with pkgs; [
          cargo
          rustc
        ];
      };
    }
  )
