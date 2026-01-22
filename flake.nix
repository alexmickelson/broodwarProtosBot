{
  description = "Broodwar Wine Bot - StarCraft Broodwar AI bot with Rust and BWAPI";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    nixpkgs-stable.url = "github:NixOS/nixpkgs/nixos-24.11";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, nixpkgs-stable, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        pkgs-stable = import nixpkgs-stable {
          inherit system;
        };

        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          targets = [ "x86_64-pc-windows-gnu" ];
          extensions = [ "rust-src" ];
        };

        mingwPkgs = pkgs.pkgsCross.mingwW64;
        mingwCC = mingwPkgs.stdenv.cc;
        
        # Get GCC version dynamically
        gccVersion = mingwPkgs.stdenv.cc.cc.version;
        
        buildInputs = with pkgs; [
          rustToolchain
          mingwCC
        ];

        nativeBuildInputs = with pkgs; [
          pkg-config
          clang
          llvmPackages.libclang
        ];

        shellEnv = {
          CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER = "${mingwCC}/bin/x86_64-w64-mingw32-gcc";
          CARGO_TARGET_X86_64_PC_WINDOWS_GNU_RUSTFLAGS = "-L ${mingwPkgs.windows.pthreads}/lib";
          CC_x86_64_pc_windows_gnu = "${mingwCC}/bin/x86_64-w64-mingw32-gcc";
          CXX_x86_64_pc_windows_gnu = "${mingwCC}/bin/x86_64-w64-mingw32-g++";
          AR_x86_64_pc_windows_gnu = "${mingwCC}/bin/x86_64-w64-mingw32-ar";
          
          LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";
          
          # Bindgen configuration for MinGW cross-compilation
          # Tell bindgen to use mingw headers, not Linux headers
          BINDGEN_EXTRA_CLANG_ARGS = pkgs.lib.concatStringsSep " " [
            "--target=x86_64-w64-mingw32"
            # Use -isystem to add includes with lower priority than -I
            # This allows the mingw headers to find clang intrinsics
            "-isystem${pkgs.llvmPackages.libclang.lib}/lib/clang/${pkgs.llvmPackages.libclang.version}/include"
            "-isystem${mingwCC.cc}/include/c++/${gccVersion}"
            "-isystem${mingwCC.cc}/include/c++/${gccVersion}/x86_64-w64-mingw32"
            "-isystem${mingwCC.cc}/include/c++/${gccVersion}/backward"
            "-isystem${mingwPkgs.windows.mingw_w64_headers}/include"
            "-isystem${mingwPkgs.windows.pthreads}/include"
            "-D_WIN32"
            "-D_WIN64"
          ];
          
          # Set target for bindgen
          TARGET = "x86_64-pc-windows-gnu";
        };

        # Build script
        buildScript = pkgs.writeShellScriptBin "build-protossbot" ''
          set -e
          cd protossbot
          cargo build --target x86_64-pc-windows-gnu --release
        '';

        buildDebugScript = pkgs.writeShellScriptBin "build-protossbot-debug" ''
          set -e
          cd protossbot
          cargo build --target x86_64-pc-windows-gnu
        '';

        cleanScript = pkgs.writeShellScriptBin "clean-protossbot" ''
          cd protossbot
          cargo clean
          echo "✓ Cleaned build artifacts"
        '';

        checkScript = pkgs.writeShellScriptBin "check-protossbot" ''
          cd protossbot
          cargo check --target x86_64-pc-windows-gnu
        '';

      in
      {
        devShells.default = pkgs.mkShell (shellEnv // {
          buildInputs = buildInputs ++ nativeBuildInputs ++ [
            buildScript
            buildDebugScript
            cleanScript
            checkScript
            
            # Additional development tools
            pkgs.cargo-watch
            pkgs.cargo-edit
            pkgs.rust-analyzer
            
            # Wine from stable nixpkgs
            pkgs-stable.wineWowPackages.stable
            
            # Script dependencies
            pkgs.unzip
            pkgs.curl
            pkgs.p7zip
            pkgs.wget
            pkgs.xorg.xorgserver
          ];

          shellHook = ''
            echo "Available commands"
            echo "  build-protossbot        - Build release version for Windows"
            echo "  build-protossbot-debug  - Build debug version for Windows"
            echo "  check-protossbot        - Quick check without building"
            echo "  clean-protossbot        - Clean build artifacts"
          '';
        });

        packages = {
          # Build the Windows executable
          protossbot = pkgs.stdenv.mkDerivation {
            pname = "protossbot";
            version = "0.1.0";
            src = ./protossbot;

            nativeBuildInputs = nativeBuildInputs ++ buildInputs;

            buildPhase = ''
              export HOME=$TMPDIR
              ${pkgs.lib.concatStringsSep "\n" 
                (pkgs.lib.mapAttrsToList 
                  (name: value: "export ${name}=\"${value}\"") 
                  shellEnv)}
              
              cargo build --release --target x86_64-pc-windows-gnu --locked
            '';

            installPhase = ''
              mkdir -p $out/bin
              cp target/x86_64-pc-windows-gnu/release/protossbot.exe $out/bin/
            '';
          };

          default = self.packages.${system}.protossbot;
        };

        apps = {
          build = {
            type = "app";
            program = "${buildScript}/bin/build-protossbot";
          };
          
          build-debug = {
            type = "app";
            program = "${buildDebugScript}/bin/build-protossbot-debug";
          };

          clean = {
            type = "app";
            program = "${cleanScript}/bin/clean-protossbot";
          };

          check = {
            type = "app";
            program = "${checkScript}/bin/check-protossbot";
          };

          default = self.apps.${system}.build;
        };
      }
    );
}
