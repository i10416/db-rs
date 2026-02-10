{
  description = "Flake to manage my Terraform workspace.";
  inputs.nixpkgs.url = "nixpkgs/nixpkgs-unstable";
  inputs.flake-parts.url = "github:hercules-ci/flake-parts";
  inputs.rust-overlay = {
    url = "github:oxalica/rust-overlay";
    inputs.nixpkgs.follows = "nixpkgs";
  };
  outputs =
    inputs@{
      self,
      nixpkgs,
      flake-parts,
      rust-overlay,
    }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "aarch64-darwin"
        "x86_64-darwin"
      ];
      perSystem =
        { pkgs, system, ... }:
        let
          overlays = [
            (import rust-overlay)
          ];
          pkgs = import nixpkgs {
            inherit system overlays;
          };
          toolchain = (pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain).override {
            extensions = [ "rust-src" ];
          };

          sharedBuildInputs = with pkgs; [
            # Rust
            cargo-nextest
            toolchain
            mold
          ];
        in
        {
          devShells = {
            default = pkgs.mkShell {
              shellHook = ''
                export PS1='\n\[\033[1;34m\][:\w]\$\[\033[0m\] '
                export TENV_AUTO_INSTALL=true;
                export PATH="$HOME/.cargo/bin:$PATH"
              '';
              buildInputs = sharedBuildInputs;
            };
          };
        };
    };
}
