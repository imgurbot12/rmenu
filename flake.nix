{
  description = "rmenu: Another customizable Application-Launcher written in Rust";

  inputs.nixpkgs.url = "nixpkgs/nixpkgs-unstable";

  outputs = {
    nixpkgs,
    self,
    ...
  }: let
    systems = [
      "x86_64-linux"
      "aarch64-linux"
    ];

    forAllSystems = f: nixpkgs.lib.genAttrs systems (system: f system);
  in {
    devShells = forAllSystems (system: let
      pkgs = nixpkgs.legacyPackages.${system};
    in {
      default = pkgs.mkShell {
        packages = with pkgs; [
          # base toolchain
          pkg-config
          cargo

          # runtime deps
          glib
          gtk3
          libsoup_3
          networkmanager
          webkitgtk_4_1     

          # nix utils
          self.formatter.${pkgs.system}
          nil
        ];
      };
    });

    formatter = forAllSystems (system: let
      pkgs = nixpkgs.legacyPackages.${system};
      in pkgs.alejandra);
    packages = forAllSystems (system: let
      pkgs = nixpkgs.legacyPackages.${system};
      rmenu-pkg = (pkgs.callPackage ./nix/package.nix {});
    in {
      rmenu = rmenu-pkg;
      default = rmenu-pkg;
    });
  };
}

