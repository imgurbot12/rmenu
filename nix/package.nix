{ cargo
, fetchFromGitHub
, glib
, gtk3
, lib
, libsoup_3
, networkmanager
, pkg-config
, rustPlatform
, rustc
, stdenv
, webkitgtk_4_1
, wrapGAppsHook
}:
rustPlatform.buildRustPackage rec {
  version = "1.1.0";
  pname = "rmenu";

  src = lib.cleanSource ../.;
  
#  src = fetchFromGitHub {
#    rev = "188f542"; # "v${version}";
#    owner = "imgurbot12";
#    repo = pname;
#    hash = "sha256-IRwYxjyHdP6794pQjyyUmofO8uakMY22pqdFkJZ5Mdo=";
#  };

  strictDeps = true;

  cargoLock = {
    lockFile = ../Cargo.lock;
    outputHashes = {
      "gio-0.19.0" = "sha256-+PAQNJ9sTk8aKAhA/PLQWDCKDT/cQ+ukdbem7g1J+pU=";
      "nm-0.4.0" = "sha256-53ipJU10ZhIKIF7PCw5Eo/e/reUK0qpyTyE7uIrCD88=";
    };
  };

  
  nativeBuildInputs = [
    pkg-config
    wrapGAppsHook
  ];

  buildInputs = [
    glib
    gtk3
    libsoup_3
    networkmanager
    webkitgtk_4_1
  ];

  postInstall = ''
    mkdir $out/themes
    mkdir $out/plugins
    cp -vfr $src/themes/* $out/themes/.
    cp -vfr $src/other-plugins/* $out/plugins/.
    mv $out/bin/* $out/plugins # everything is a plugin by default

    mv $out/plugins/rmenu $out/bin/rmenu
    mv $out/plugins/rmenu-build $out/bin/rmenu-build

    mkdir -p $out/etc/rmenu
    cp -vf $src/rmenu/public/config.yaml $out/etc/rmenu/config.yaml
    ln -sf  $out/themes/dark.css $out/etc/rmenu/style.css
  '';

  doCheck = true;

  meta = with lib; {
    changelog = "https://github.com/imgurbot12/rmenu/commits/master/";
    description = "Another customizable Application-Launcher written in Rust ";
    homepage = "https://github.com/imgurbot12/rmenu";
    mainProgram = "rmenu";
    maintainers = [ maintainers.grimmauld ];
    platforms = platforms.linux;
  };
}
