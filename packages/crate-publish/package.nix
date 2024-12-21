{
  cargo,
  gcc,
  rustc,
  toml-cli,
  writeShellApplication,
}:
writeShellApplication {
  name = "crate-publish";
  runtimeInputs = [
    cargo
    gcc
    rustc
    toml-cli
  ];
  text = builtins.readFile ./script.sh;
}
