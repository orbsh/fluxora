{
  description = "Fluxora development environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs = { self, nixpkgs }:
    let
      system = "x86_64-linux";
      pkgs = nixpkgs.legacyPackages.${system};
    in {
      devShells.${system}.default = pkgs.mkShell {
        buildInputs = with pkgs; [
          # C/C++ toolchain
          gcc
          cmake
          pkg-config

          # rdkafka-sys dependencies
          perl
          curl.dev
          openssl.dev
          zstd.dev
          lz4.dev
          cyrus_sasl.dev
          zlib.dev
        ];

        shellHook = ''
          export OPENSSL_DIR="${pkgs.openssl.dev}"
          export OPENSSL_LIB_DIR="${pkgs.openssl.out}/lib"
          export CURL_INCLUDE_DIR="${pkgs.curl.dev}/include"
          export CURL_LIB_DIR="${pkgs.curl.out}/lib"
          export ZSTD_INCLUDE_DIR="${pkgs.zstd.dev}/include"
          export ZSTD_LIB_DIR="${pkgs.zstd.out}/lib"
          export LZ4_INCLUDE_DIR="${pkgs.lz4.dev}/include"
          export LZ4_LIB_DIR="${pkgs.lz4.out}/lib"
          export CPATH="${pkgs.curl.dev}/include:${pkgs.openssl.dev}/include:${pkgs.zstd.dev}/include:${pkgs.lz4.dev}/include:${pkgs.zlib.dev}/include:$CPATH"
          export LIBRARY_PATH="${pkgs.curl.out}/lib:${pkgs.openssl.out}/lib:${pkgs.zstd.out}/lib:${pkgs.lz4.out}/lib:${pkgs.zlib.out}/lib:$LIBRARY_PATH"
        '';
      };
    };
}
