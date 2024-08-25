let
  pkgs = import <nixpkgs> {};

  # Rolling updates, not deterministic.
  # pkgs = import (fetchTarball("channel:nixpkgs-unstable")) {};
in pkgs.mkShell rec {
  buildInputs = with pkgs; [ 
    cargo 
    rustc 
    rust-analyzer
    libxkbcommon
    libGL

    # WINIT_UNIX_BACKEND=wayland
    wayland

    # WINIT_UNIX_BACKEND=x11
    xorg.libXcursor
    xorg.libXrandr
    xorg.libXi
    xorg.libX11
  ];
  
  LD_LIBRARY_PATH = "${pkgs.lib.makeLibraryPath buildInputs}";
}
