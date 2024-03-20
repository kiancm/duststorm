{
  description = "duststorm dev environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    maelstromPkg.url = "path:./maelstrom";
  };

  outputs = { self, nixpkgs, maelstromPkg, ... }:
    let
      system = "x86_64-linux";
      pkgs = nixpkgs.legacyPackages.${system};
      maelstrom = maelstromPkg.packages.${system}.default;
    in
    {
      devShells.${system}.default = pkgs.mkShell {
        nativeBuildInputs = [ 
          pkgs.gcc
          pkgs.vscode-extensions.llvm-org.lldb-vscode
          pkgs.bacon
          pkgs.openjdk21
          maelstrom
        ];
      };
    };
}
