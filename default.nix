with import <nixpkgs> {};

let
  rust = (rustChannels.stable.rust.override {
    targets = [
      "x86_64-unknown-linux-musl"
    ];
  });
  in {

  airstation = rustPlatform.buildRustPackage rec {
    name = "airstation-${version}";
    version = "0.1.0";
    src = ./.;
    cargoSha256 = "1d6b8jglwk2g8k6xxzf1804ndmsk8q6qry9hgb63fcih55dzkn74";
  };

  shell = pkgs.mkShell {
    name = "env-airstation";
    buildInputs = [
      rust
      screen
    ];

    PKG_CONFIG_ALLOW_CROSS=true;
    PKG_CONFIG_ALL_STATIC=true;
    LIBZ_SYS_STATIC=1;

    OPENSSL_STATIC=1;
    OPENSSL_DIR = pkgsStatic.openssl.dev;
    OPENSSL_LIB_DIR = "${pkgsStatic.openssl.out}/lib";
  };

}
