inputs:
let
  inherit (builtins)
    attrNames
    readDir
    ;

  inherit (inputs.nixpkgs)
    lib
    ;

  inherit (lib.attrsets)
    filterAttrs
    ;
in
{
  getDirectories =
    path: attrNames (filterAttrs (_: fileType: fileType == "directory") (readDir path));
}
