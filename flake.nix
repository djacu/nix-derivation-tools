{
  description = "nix-derivation-parser";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    treefmt-nix.inputs.nixpkgs.follows = "nixpkgs";
    treefmt-nix.url = "github:numtide/treefmt-nix";
  };

  outputs = inputs: {
    checks = import ./checks inputs;
    devShells = import ./dev-shells inputs;
    formatter = import ./formatter inputs;
    formatterModule = import ./formatter-module inputs;
    legacyPackages = import ./legacy-packages inputs;
    library = import ./library inputs;
    overlays = import ./overlays inputs;
    packages = import ./packages inputs;
  };
}
