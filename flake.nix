{
  description = "Krait development environment with Rust toolchains";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
      in
      {
        devShells.default = pkgs.mkShell {
          nativeBuildInputs = [ 
            # Must have rust-src for -Z build-std
            (pkgs.rust-bin.nightly.latest.default.override {
              extensions = [ "rust-src" ];
            })
          ];

          buildInputs = [
            # Rust toolchains
            pkgs.rust-bin.nightly.latest.default
            pkgs.rust-analyzer

            # eBPF and kernel development tools
            pkgs.llvmPackages.libllvm # for llvm-objdump
            pkgs.bpftools
            pkgs.bpf-linker

            # WireGuard tools
            pkgs.wireguard-tools

            # General development tools
            pkgs.pkg-config
            pkgs.openssl
          ];

          shellHook = ''
            export RUSTC_BOOTSTRAP=1

            echo "Krait development environment"
            echo ""
            echo "eBPF tools available: bpftool, clang, llvm, bpf-linker"
            echo "WireGuard tools: wg, wg-quick"
          '';

          # Environment variables for eBPF development
          LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";
          BINDGEN_EXTRA_CLANG_ARGS = "-I${pkgs.glibc.dev}/include";
        };
      });
}
