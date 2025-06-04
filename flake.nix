{
  # Source: https://github.com/the-nix-way/dev-templates/blob/main/rust-toolchain/flake.nix
  description = "A Nix-flake-based Rust development environment for Pop";

  inputs = {
    nixpkgs.url = "https://flakehub.com/f/NixOS/nixpkgs/0.1";
    fenix = {
      url = "https://flakehub.com/f/nix-community/fenix/0.1";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = inputs:
    let
      supportedSystems = [ "x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin" ];
      forEachSupportedSystem = f: inputs.nixpkgs.lib.genAttrs supportedSystems (system: f {
        pkgs = import inputs.nixpkgs {
          inherit system;
          overlays = [
            inputs.self.overlays.default
          ];
        };
      });
    in
    {
      overlays.default = final: prev: {
        rustToolchain = inputs.fenix.packages.${prev.stdenv.hostPlatform.system}.fromToolchainFile
          {
            file = ./rust-toolchain.toml;
            sha256 = "sha256-X/4ZBHO3iW0fOenQ3foEvscgAPJYl2abspaBThDOukI=";
          };
        rustfmt-nightly = inputs.fenix.packages.${prev.stdenv.hostPlatform.system}.latest.rustfmt;
      };

      devShells = forEachSupportedSystem ({ pkgs }: {
        default = pkgs.mkShellNoCC {
          packages = with pkgs; [
            protobuf
            rustToolchain
            rustfmt-nightly
          ];

          shellHook = ''
            # Use xcode command line tools
            export PATH="$(echo $PATH | sd "${pkgs.xcbuild.xcrun}/bin" "")"
            unset DEVELOPER_DIR
            export SDKROOT="/Library/Developer/CommandLineTools/SDKs/MacOSX.sdk"
            # Required by rust-analyzer
            export RUST_SRC_PATH="${pkgs.rustToolchain}/lib/rustlib/src/rust/library"
          '';
        };
        formatter = pkgs.nixfmt-rfc-style;
      });
    };
}
