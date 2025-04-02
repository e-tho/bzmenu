{
  description = "Menu-driven Bluetooth management interface for Linux";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      rust-overlay,
      flake-utils,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [ (import rust-overlay) ];

        pkgs = import nixpkgs {
          inherit system overlays;
        };

        cargoToml = builtins.fromTOML (builtins.readFile ./Cargo.toml);
        cargoPackageVersion = cargoToml.package.version;

        commitHash = self.shortRev or self.dirtyShortRev or "unknown";

        version = "${cargoPackageVersion}-${commitHash}";
      in
      {
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "bzmenu";
          inherit version;

          src = ./.;

          cargoLock = {
            lockFile = ./Cargo.lock;
          };

          nativeBuildInputs = with pkgs; [
            pkg-config
          ];

          buildInputs = with pkgs; [
            dbus.dev
          ];

          doCheck = true;
          CARGO_BUILD_INCREMENTAL = "false";
          RUST_BACKTRACE = "full";

          meta = {
            description = "Menu-driven Bluetooth management interface for Linux";
            homepage = "https://github.com/e-tho/bzmenu";
            license = pkgs.lib.licenses.gpl3;
            maintainers = [
              {
                github = "e-tho";
              }
            ];
            mainProgram = "bzmenu";
          };
        };

        devShells.default =
          with pkgs;
          mkShell {
            nativeBuildInputs = [
              pkg-config
              (rust-bin.stable.latest.default.override {
                extensions = [ "rust-src" ];
              })
            ];

            buildInputs = [
              dbus.dev
            ];

            inherit (self.packages.${system}.default) CARGO_BUILD_INCREMENTAL RUST_BACKTRACE;
          };
      }
    );
}
