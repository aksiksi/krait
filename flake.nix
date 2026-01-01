{
  description = "Krait eBPF network security system";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
        };
        commonBuildInputs = [
          pkgs.llvmPackages.libllvm
          pkgs.bpftools
          pkgs.bpf-linker
          pkgs.pkg-config
          pkgs.openssl
        ];
      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = commonBuildInputs ++ [
            pkgs.rust-analyzer
            pkgs.rustup
            pkgs.wireguard-tools
          ];

          shellHook = ''
            export RUSTC_BOOTSTRAP=1

            # Install nightly Rust if not already available
            if ! rustup toolchain list | grep -q nightly; then
              echo "Installing Rust nightly toolchain..."
              rustup toolchain install nightly
            fi

            # Install rust-src component for eBPF compilation
            rustup component add --toolchain nightly rust-src

            # Set nightly as default for this shell
            rustup default nightly

            rustup target add x86_64-unknown-linux-musl

            echo "Krait development environment"
            echo "Rust toolchain: $(rustc --version)"
            echo ""
            echo "Build commands:"
            echo "  cargo build -p krait-ebpf --target bpfel-unknown-none -Z build-std=core --release"
            echo "  cargo build -p krait-agent --target x86_64-unknown-linux-musl"
          '';

          # Environment variables for development
          LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";
          BINDGEN_EXTRA_CLANG_ARGS = "-I${pkgs.glibc.dev}/include";
        };
      });
}

