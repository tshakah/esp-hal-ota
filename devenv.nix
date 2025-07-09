{
  pkgs,
  inputs,
  ...
}: {
  cachix.enable = false;

  packages = with pkgs; [nil alejandra inotify-tools cargo-outdated];

  languages.nix.enable = true;
  languages.rust = {
    enable = true;
    channel = "nightly";
  };

  enterTest = ''
    cargo build
    cargo build --no-default-features --features rustler
  '';

  git-hooks.hooks = {
    alejandra.enable = true;
    rustfmt.enable = true;
  };
}
