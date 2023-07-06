{
  perSystem = { config, self', pkgs, lib, ... }:
    let
      src = lib.sourceFilesBySuffices ./. [ ".rs" ".toml" "Cargo.lock" ];
      inherit (lib.importTOML (src + "/Cargo.toml")) package;
    in
    {
      dream2nix.inputs."backend" = {
        source = src;
        projects."backend" = { name, ... }: {
          inherit name;
          subsystem = "rust";
          translator = "cargo-lock";
        };
      };

      packages = config.dream2nix.outputs."backend".packages;
      devShells.backend = pkgs.mkShell {
        inherit (package) name;
        inputsFrom = [
          config.dream2nix.outputs."backend".devShells.${package.name}
        ];
        packages = with pkgs; [
          rust-analyzer
          nil
        ];
      };
    };
}
