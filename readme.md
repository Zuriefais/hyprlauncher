<div align='center'>

<h2>Hyprlauncher <img src='https://raw.githubusercontent.com/hyprutils/.github/refs/heads/main/hyprutils_transparent.png'width='18' height='18'></h2>

An unofficial GUI for launching applications, built with GTK4 and Rust. 🚀🦀<br>

![Preview](.github/preview.png)

</div>

## Installation

### Arch Linux
There are 2 different [AUR](https://aur.archlinux.org) packages available:

- [hyprlauncher](https://aur.archlinux.org/packages/hyprlauncher) - Latest release built from source
- [hyprlauncher-bin](https://aur.archlinux.org/packages/hyprlauncher-bin) - Latest release in binary form

Install the preferred package with:
```bash
git clone https://aur.archlinux.org/<package>.git
cd <package>
makepkg -si
```

Or, if you're using an [AUR Helper](https://wiki.archlinux.org/title/AUR_helpers), it's even simpler (using [paru](https://github.com/Morganamilo/paru) as an example):
```bash
paru -S <package>
```

## Building from source
1. Install Rust (preferably `rustup`) through your distro's package or [the official script](https://www.rust-lang.org/tools/install)
2. Install `git`, `pango` and `gtk4`
3. Clone this repository:
`git clone https://github.com/nnyyxxxx/hyprlauncher && cd hyprlauncher`
4. Compile the app with `cargo build --release` or run it directly with `cargo run --release`

## Credits:
- [Zuriefais](https://github.com/Zuriefais) - My own fork
- [Nyx](https://github.com/nnyyxxxx) - Implementing the GUI
- [Adam](https://github.com/adamperkowski) - Code improvements
- [Vaxry](https://github.com/vaxerski) - Hyprland
- [rust-gtk](https://github.com/gtk-rs/gtk4-rs) - The GTK4 library
- [Hyprland](https://github.com/hyprwm/Hyprland) - The window manager

<h6 align='center'>Copyright (C) 2024 HyprUtils<h6>
