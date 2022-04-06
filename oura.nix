{ stdenv, lib, fetchurl }:
let

  version = "1.3.0";

  rpath = lib.makeLibraryPath [ ];

  src = fetchurl {
    url = "https://github.com/txpipe/oura/releases/download/v${version}/oura-x86_64-unknown-linux-gnu.tar.gz";
    sha256 = "sha256-HLL/6/9oWpaMsZ0lp4IS0c/wE4zVM5+ZcEceVP+XFgs=";
  };

in stdenv.mkDerivation {
  name = "oura-${version}";

  system = "x86_64-linux";

  inherit src;

  nativeBuildInputs = [ ];
  buildInputs = [ ];
  
  unpackPhase = "true";

  installPhase = ''
    mkdir -p $out/bin
    tar -xf $src -C $out/bin
  '';

  postFixup = ''
    patchelf --set-interpreter "$(cat $NIX_CC/nix-support/dynamic-linker)" "$out/bin/oura" || true
    patchelf --set-rpath ${rpath} "$out/bin/oura" || true
  '';

  meta = with lib; {
    description = "Oura: The tail of Cardano";
    homepage = https://txpipe.github.io/oura/;
    license = licenses.unfree;
    maintainers = with lib.maintainers; [ ];
    platforms = [ "x86_64-linux" ];
  };
}
