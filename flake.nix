{
  description = "A flake to build a bare-metal RISC-V Rust project";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs";

  outputs = {
    self,
    nixpkgs,
  }: let
    system = "aarch64-darwin";
    pkgs = nixpkgs.legacyPackages.${system};
    riscv-toolchain = import nixpkgs {
      localSystem = "${system}";
      crossSystem = {
        config = "riscv32-linux";
      };
    };
  in {
    devShells.${system}.default = pkgs.mkShell {
      buildInputs = [
        pkgs.rustup
        pkgs.rust-analyzer
        pkgs.cargo
        pkgs.cmake
        pkgs.gcc
        pkgs.binutils
        pkgs.qemu
      ];
      shellHook = ''
        if ! rustup toolchain list | grep -q "^nightly"; then
          echo "Installing nightly toolchain..."
          rustup install nightly
        fi

        rustup override set nightly
        rustup component add rust-analyzer
      rustup +nightly component add miri

        # Set nightly as default if not already
        if ! rustup show active-toolchain 2>/dev/null | grep -q "^nightly"; then
          echo "Setting nightly as default..."
          rustup default nightly
        fi

        # Install RISC-V target only if missing
        if ! rustup target list --installed --toolchain nightly | grep -q "riscv32im-unknown-none-elf"; then
          echo "Adding RISC-V target..."
          rustup target add riscv32im-unknown-none-elf --toolchain nightly
        fi

        echo "Ready to build for bare-metal RISC-V!"
      '';
    };
  };
}
