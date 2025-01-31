{ pkgs ? import (fetchTarball "https://releases.nixos.org/nixos/24.11/nixos-24.11.713818.59e618d90c06/nixexprs.tar.xz") {} }: # #"https://nixos.org/channels/nixos-unstable/nixexprs.tar.xz"
#export PATH=$PATH:/nix/var/nix/profiles/default/bin/
pkgs.rustPlatform.buildRustPackage rec {
  pname = "off_image_relay";
  version = "0.1.0";

  src = ./.;

  cargoLock = {
    lockFile = ./Cargo.lock;
    outputHashes = {
      "cjstream-0.0.1" = "sha256-BkT+wqCG6jW+MdzfVgBXGXuK5uloTJj4NqAn0368clY=";
    };
  };

  buildInputs = with pkgs; [
    openssl
    llvmPackages_19.libclang
  ];

  nativeBuildInputs = with pkgs; [
    pkg-config
    llvmPackages_19.libclang
    rustPlatform.bindgenHook
  ];

  LIBCLANG_PATH = "${pkgs.llvmPackages_19.libclang.lib}/lib";
}
