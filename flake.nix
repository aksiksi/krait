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
          buildInputs = with pkgs; [
            # Rust toolchains
            rust-bin.stable.latest.default
            rust-bin.nightly.latest.default

            # eBPF and kernel development tools
            bpftools
            bpf-linker

            # WireGuard tools
            wireguard-tools

            # General development tools
            pkg-config
            openssl
            cargo-generate
          ];

          shellHook = ''
            echo "Krait development environment"
            echo "Available Rust toolchains:"
            echo "  - Stable: $(rustc --version)"
            echo "  - Nightly: $(rustc +nightly --version 2>/dev/null || echo 'Use with: cargo +nightly')"
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
