{
  perSystem = { self', pkgs, lib, ... }:
    let
      src = lib.sourceFilesBySuffices ./. [ ".rs" ".toml" "Cargo.lock" ];
      inherit (lib.importTOML (src + "/Cargo.toml")) package;
    in
    {
      packages = {
        ${package.name} = pkgs.rustPlatform.buildRustPackage {
          pname = package.name;
          inherit (package) version;
          inherit src;
          cargoLock.lockFile = (src + "/Cargo.lock");
        };
        default = self'.packages.${package.name};
      };

      devShells.backend = pkgs.mkShell {
        inherit (package) name;
        inputsFrom = [ self'.packages.${package.name} ];
        packages = with pkgs; [
          rust-analyzer
          nil
        ];
      };
    };
}
