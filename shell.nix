# run with: nix-shell <filename>.nix
let
  pkgs = import <nixpkgs> { };
in
pkgs.mkShell {
  # [rust]:
  hardeningDisable = [ "all" ];
  RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";

  # [graphics]: runtime libs for winit (windowing) and wgpu (Vulkan).
  # These are dlopen'd at runtime, not linked at build time, so they must be
  # on LD_LIBRARY_PATH. The GPU driver itself (ICD) comes from the system
  # config (hardware.graphics.enable), not from this shell.
  LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath (with pkgs; [
    vulkan-loader # libvulkan.so.1 for wgpu
    libxkbcommon # keyboard handling, needed on both Wayland and X11
    wayland # libwayland-client / libwayland-cursor
    libX11 # X11 fallback path
    libXcursor
    libXi
    libXrandr
  ]);

  packages = with pkgs; [
    zsh

    # [rust]:
    rustc
    cargo
    rustfmt
    clippy
    bacon

    # [graphics]: vulkaninfo / vkcube for diagnosing driver issues
    vulkan-tools
  ];

  shellHook = ''
    # [rust]:
    export NIX_ENFORCE_PURITY=0

    # zsh terminal forwarding:
    export SHELL=${pkgs.zsh}/bin/zsh
    export ZDOTDIR="$(pwd)/.zshrc.d"
    mkdir -p "$ZDOTDIR"
    cat > "$ZDOTDIR/.zshrc" <<'EOF'
      # Source your original config
      [[ -f ~/.zshrc ]] && source ~/.zshrc

      # Prepend plain white (nix-shell) to the existing prompt
      PROMPT="%F{white}(nix-shell)%f $PROMPT"
    EOF
    exec ${pkgs.zsh}/bin/zsh
  '';
}
