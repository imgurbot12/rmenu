{ lib, pkgs, ... }:
let
  rmenu-pkg = pkgs.callPackage ./package.nix {};
in {
  config.environment.systemPackages = [ rmenu-pkg ];

  meta.maintainers = with lib.maintainers; [ grimmauld ];
}
