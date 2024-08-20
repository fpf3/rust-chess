let
  # Pinned nixpkgs, deterministic. Last updated: 2/12/21.
  pkgs = import (fetchTarball("https://github.com/NixOS/nixpkgs/archive/63dacb46bf939521bdc93981b4cbb7ecb58427a0.tar.gz")) {};

  # Rolling updates, not deterministic.
  # pkgs = import (fetchTarball("channel:nixpkgs-unstable")) {};
in pkgs.mkShell rec {
  buildInputs = with pkgs; [ 
    cargo 
    rustc 
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
