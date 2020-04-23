{ pkgs ? import <nixpkgs> {} }:
  pkgs.mkShell {
    buildInputs = [ pkgs.openssl pkgs.git pkgs.docker pkgs.gitAndTools.pre-commit pkgs.cargo pkgs.cargo-tree pkgs.httpie pkgs.dhall pkgs.dhall-json ];
    shellHook = ''
      pre-commit install;
    '';
}
