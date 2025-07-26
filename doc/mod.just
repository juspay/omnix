default:
  @just --list doc

# Run emanote live server
run:
  nix run

# Do link checks on docs
check:
    nix run .#linkCheck
