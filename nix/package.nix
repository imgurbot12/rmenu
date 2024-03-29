{ cargo
, fetchFromGitHub
, glib
, gnused
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
    gnused
  ];

  postInstall = ''
    # copy themes and plugins
    mkdir $out/themes
    mkdir $out/plugins
    cp -vfr $src/themes/* $out/themes/.
    cp -vfr $src/other-plugins/* $out/plugins/.
    mv $out/bin/* $out/plugins # everything is a plugin by default

    # rmenu and rmenu-build are actual binaries
    mv $out/plugins/rmenu $out/bin/rmenu
    mv $out/plugins/rmenu-build $out/bin/rmenu-build

    # fix plugin names
    # desktop  network  pactl-audio.sh  powermenu.sh  run  window
    mv $out/plugins/run $out/plugins/rmenu-run
    mv $out/plugins/desktop $out/plugins/rmenu-desktop
    mv $out/plugins/network $out/plugins/rmenu-network
    mv $out/plugins/window $out/plugins/rmenu-window

    # fix config and theme
    mkdir -p $out/etc/xdg/rmenu
    cp -vf $src/rmenu/public/config.yaml $out/etc/xdg/rmenu/config.yaml
    sed -i "s@~\/\.config\/rmenu\/themes@$out\/themes@g" $out/etc/xdg/rmenu/config.yaml
    sed -i "s@~\/\.config\/rmenu@$out\/plugins@g" $out/etc/xdg/rmenu/config.yaml
    ln -sf  $out/themes/dark.css $out/etc/xdg/rmenu/style.css
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
