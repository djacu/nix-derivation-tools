{
  description = "nix-derivation-parser";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
  };

  outputs = inputs: {
    devShells = import ./dev-shells inputs;
    legacyPackages = import ./legacy-packages inputs;
    library = import ./library inputs;
    overlays = import ./overlays inputs;
    packages = import ./packages inputs;
  };
}
