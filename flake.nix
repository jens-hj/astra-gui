{
  description = "Astra GUI - Graphics backend agnostic UI library";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = {
    self,
    nixpkgs,
    rust-overlay,
    flake-utils,
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        overlays = [(import rust-overlay)];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        # Detect if we're on macOS
        isDarwin = pkgs.stdenv.isDarwin;

        # Rust toolchain
        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = ["rust-src" "rust-analyzer"];
        };

        # Platform-specific dependencies for wgpu
        linuxInputs = with pkgs; [
          udev
          vulkan-loader
          xorg.libX11
          xorg.libxcb
          xorg.libXcursor
          xorg.libXi
          xorg.libXrandr
          libxkbcommon
          wayland
        ];

        darwinInputs = with pkgs; [
          libiconv
        ];

        buildInputs =
          if isDarwin
          then darwinInputs
          else linuxInputs;

        nativeBuildInputs = with pkgs; [
          pkg-config
          clang
          lld
        ];
      in {
        devShells.default = pkgs.mkShell {
          inherit buildInputs nativeBuildInputs;

          packages = with pkgs; [
            rustToolchain
            cargo-watch
          ];

          # Environment variables for wgpu (Linux only)
          LD_LIBRARY_PATH =
            if isDarwin
            then ""
            else pkgs.lib.makeLibraryPath buildInputs;

          shellHook = ''
            echo "Astra GUI development environment"
            echo "Rust version: $(rustc --version)"
          '';
        };
      }
    );
}
