<div align="center">
  <h1>bzmenu</h1>
</div>

## About

`bzmenu` (**B**lue**Z** **Menu**) allows using your menu of choice to manage Bluetooth.

## Prerequisites

[BlueZ](https://www.bluez.org) must be installed, along with either a supported launcher or any `stdin` mode launcher.

> [!NOTE]
> To ensure proper icon display, you can either install [nerdfonts](https://www.nerdfonts.com) for font-based icons (usage is optional) or use the `--icon xdg` flag for image-based icons from your XDG theme.

### Compatibility

- [Fuzzel](https://codeberg.org/dnkl/fuzzel)
- [Rofi](https://github.com/davatorium/rofi)
- [Wofi](https://hg.sr.ht/~scoopta/wofi)
- [dmenu](https://tools.suckless.org/dmenu)

Use `custom` mode if your launcher is not supported.

## Installation

### Build from source

Run the following commands:

```shell
git clone https://github.com/e-tho/bzmenu
cd bzmenu
cargo build --release
```

An executable file will be generated at `target/release/bzmenu`, which you can then copy to a directory in your `$PATH`.

### Nix

Add the flake as an input:

```nix
inputs.bzmenu.url = "github:e-tho/bzmenu";
```

Install the package:

```nix
{ inputs, ... }:
{
  environment.systemPackages = [ inputs.bzmenu.packages.${pkgs.system}.default ];
}
```

## Usage

### Supported menus

Specify an application using `-m` or `--menu` flag.

```shell
bzmenu -m fuzzel
```

### Custom menus

Specify `custom` as the menu and set your command using the `--menu-command` flag. Ensure your launcher supports `stdin` mode, and that it is properly configured in the command.

```shell
bzmenu -m custom --menu-command "my_custom_launcher --flag"
```

#### Prompt and Placeholder support

Use either `{prompt}` or `{placeholder}` as the value for the relevant flag in your command; each will be replaced with the appropriate text as needed. They return the same string, with `{prompt}` adding a colon at the end.

```shell
bzmenu -m custom --menu-command "my_custom_launcher --prompt-flag '{prompt}'" # or --placeholder-flag '{placeholder}'
```

#### Example to enable all features

This example demonstrates enabling all available features in custom mode with `fuzzel`.

```shell
bzmenu -m custom --menu-command "fuzzel -d -p '{prompt}'"
```

### Available Options

| Flag             | Description                                           | Supported Values                            | Default Value |
| ---------------- | ----------------------------------------------------- | ------------------------------------------- | ------------- |
| `-m`, `--menu`   | Specify the menu application to use.                  | `dmenu`, `rofi`, `wofi`, `fuzzel`, `custom` | `dmenu`       |
| `--menu-command` | Specify the command to use when `custom` menu is set. | Any valid shell command                     | `None`        |
| `-i`, `--icon`   | Specify the icon type to use.                         | `font`, `xdg`                               | `font`        |
| `-s`, `--spaces` | Specify icon to text space count (font icons only).   | Any positive integer                        | `1`           |

## License

GPLv3
