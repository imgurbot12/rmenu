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
      overrides = (builtins.fromTOML (builtins.readFile ./rust-toolchain.toml));
      libPath = with pkgs; lib.makeLibraryPath [
        # load external libraries that you need in your rust project here
        glib
        gtk3
        libsoup_3
        networkmanager
        webkitgtk_4_1
      ];
    in {
      default = pkgs.mkShell {
        buildInputs = with pkgs; [
        llvmPackages.bintools
        rustup
      ];
      RUSTC_VERSION = overrides.toolchain.channel;
      # https://github.com/rust-lang/rust-bindgen#environment-variables
      LIBCLANG_PATH = pkgs.lib.makeLibraryPath [ pkgs.llvmPackages_latest.libclang.lib ];
      shellHook = ''
        export PATH=$PATH:''${CARGO_HOME:-~/.cargo}/bin
        export PATH=$PATH:''${RUSTUP_HOME:-~/.rustup}/toolchains/$RUSTC_VERSION-x86_64-unknown-linux-gnu/bin/
      '';

      # Add precompiled library to rustc search path
      RUSTFLAGS = (builtins.map (a: ''-L ${a}/lib'') [
        # add libraries here (e.g. pkgs.libvmi)
      ]);
      LD_LIBRARY_PATH = libPath;
      # Add glibc, clang, glib, and other headers to bindgen search path
      BINDGEN_EXTRA_CLANG_ARGS =
      # Includes normal include path
      (builtins.map (a: ''-I"${a}/include"'') [
        # add dev libraries here (e.g. pkgs.libvmi.dev)
        pkgs.glibc.dev
      ])
      # Includes with special directory paths
      ++ [
        ''-I"${pkgs.llvmPackages_latest.libclang.lib}/lib/clang/${pkgs.llvmPackages_latest.libclang.version}/include"''
        ''-I"${pkgs.glib.dev}/include/glib-2.0"''
        ''-I${pkgs.glib.out}/lib/glib-2.0/include/''
      ];
        packages = with pkgs; [
          # base toolchain
          pkg-config
          rustup 

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

    nixosModules = let
      rmenu-module = (import ./nix/rmenu.nix);
    in {
      rmenu = rmenu-module;
      default = rmenu-module;
    };
  };
}

