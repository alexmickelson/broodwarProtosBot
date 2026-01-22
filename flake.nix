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
          CARGO_TARGET_X86_64_PC_WINDOWS_GNU_RUSTFLAGS = "-L ${mingwPkgs.windows.pthreads}/lib -C link-args=-static-libgcc -C link-args=-static-libstdc++";
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

        startScript = pkgs.writeShellScriptBin "start" ''
          set -e

          SCRIPT_DIR="$(pwd)"
          SCRIPTS_PATH="$SCRIPT_DIR/scripts"

          export WINEPREFIX="$SCRIPT_DIR/.wine"
          export WINEARCH=win64
          export DISPLAY=:0
          export WINEDLLOVERRIDES="mscoree,mshtml="
          export WINEDEBUG=-all
          # Add MinGW DLLs to Wine's search path
          export WINEDLLPATH="${mingwPkgs.windows.pthreads}/bin:${mingwCC.cc}/x86_64-w64-mingw32/lib"

          # Cleanup function to ensure processes are killed on exit
          cleanup() {
              echo ""
              echo "Cleaning up processes..."
              if [ -n "$XVFB_PID" ] && kill -0 $XVFB_PID 2>/dev/null; then
                  echo "Stopping Xvfb..."
                  kill $XVFB_PID 2>/dev/null || true
              fi
              if [ -n "$BOT_PID" ] && kill -0 $BOT_PID 2>/dev/null; then
                  echo "Stopping protossbot..."
                  kill $BOT_PID 2>/dev/null || true
              fi
              killall StarCraft.exe 2>/dev/null || true
              echo "Cleanup complete."
          }

          # Register cleanup function to run on script exit (success or failure)
          trap cleanup EXIT

          if [ ! -d "$WINEPREFIX" ]; then
              wine wineboot --init
          fi

          echo "Starting Xvfb virtual display..."
          Xvfb :0 -auth ~/.Xauthority -screen 0 640x480x24 > /dev/null 2>&1 &
          XVFB_PID=$!

          cd scripts
              ./4-configure-bwapi.sh
          cd ..

          echo "Building protossbot..."
          build-protossbot-debug
          echo "Starting protossbot..."
          cd "$SCRIPT_DIR/protossbot"

          RUST_BACKTRACE=1 RUST_BACKTRACE=full wine target/x86_64-pc-windows-gnu/debug/protossbot.exe &
          BOT_PID=$!
          echo "protossbot started (PID: $BOT_PID)"


          echo "Launching StarCraft with BWAPI via Chaoslauncher..."
          cd "$SCRIPT_DIR/starcraft/BWAPI/Chaoslauncher"
          wine Chaoslauncher.exe

          echo "StarCraft closed."
        '';

      in
      {
        devShells.default = pkgs.mkShell (shellEnv // {
          buildInputs = buildInputs ++ nativeBuildInputs ++ [
            buildScript
            buildDebugScript
            cleanScript
            checkScript
            startScript
            
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
            echo "  start                   - Run the bot with StarCraft"
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

          start = {
            type = "app";
            program = "${startScript}/bin/start";
          };

          default = self.apps.${system}.build;
        };
      }
    );
}
