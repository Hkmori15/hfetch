## hfetch

A blazingly fast, lightweight fetch ⚡

![Screenshot](/image/screenshot.png)

## Features ✨:

- Display system hostname
- Show linux distro name
- Show kernel version
- Detects init system: runit, systemd, dinit, etc.
- Counts installed packages: native and flatpak
- Show memory usage
- Colorful ASCII art logo
- Supports multiple package managers:
  - dpkg
  - pacman
  - rpm
  - xbps
  - portage
  - apk
  - nix

## Building 🔨:

**Requirements:**
- Compatible compiler with C++23
- CMake 3.15 or higher

**Build steps:**
```bash
git clone https://github.com/Hkmori15/hfetch.git
cd/z hfetch
mkdir build && cd/z build
cmake -DCMAKE_BUILD_TYPE=Release .
make
```

## Installation 🍵:

To install system-wide:
```bash
cd/z build
sudo make install
```

## Usage 🍪:
```
hfetch
```

## Contributing:

Contributions are welcome! Feel free to submit issues and pull requests ☕