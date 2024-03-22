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
          (pkgs.writeShellApplication {
            name = "echo-test";
            text = ''
              cargo build && maelstrom test -w echo --bin target/debug/echo --node-count 1 --time-limit 10
            '';
          })
          (pkgs.writeShellApplication {
            name = "broadcast-single-test";
            text = ''
              cargo build && maelstrom test -w broadcast --bin target/debug/broadcast --node-count 1 --time-limit 10
            '';
          })
          (pkgs.writeShellApplication {
            name = "broadcast-test";
            text = ''
              cargo build && maelstrom test -w broadcast --bin target/debug/broadcast --node-count 5 --time-limit 20 --rate 10
            '';
          })
        ];
      };
    };
}
