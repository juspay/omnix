# nix eval --raw --impure --expr 'builtins.toFile "demo.json" (builtins.toJSON (import ./demo.nix))' | xargs cat | jq
[
  {
    name = "username";
    description = "Your unix username";
    placeholder = "runner";
  }
  {
    name = "include-neovim";
    description = "Include NeoVIM configuration?";
    paths = [ "neovim.nix" ];
    value = false;
  }
  {
    name = "git-name";
    description = "Your full name";
    placeholder = "John Doe";
  }
  {
    name = "git-email";
    description = "Your email address";
    placeholder = "john.doe@gmail.com";
  }
]
