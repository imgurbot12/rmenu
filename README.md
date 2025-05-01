## RMenu

Another customizable Application-Launcher written in Rust

### Features

- Blazingly Fast 🔥
- Simple and Easy to Use
- Customizable (Configuration and CSS-Styling)
- Plugin Support
- Dmenu-Like Stdin Menu Generation

### Installation

##### Nix

RMenu now has Nix support thanks to
[@LordGrimmauld](https://github.com/LordGrimmauld)!

Try it on nix directly via

```bash
nix run github:imgurbot12/rmenu -- -r drun
```

Rmenu v1.2.0 is now in nix unstable!
Install it via:

```bash
sudo nix-channel --add https://nixos.org/channels/nixpkgs-unstable unstable
sudo nix-channel --update
sudo nix-env -iA unstable.rmenu
```

##### Other Platforms:

Install Dependencies (Ubuntu)

```bash
sudo apt install pkg-config libglib2.0-dev libghc-gi-gdk-dev libsoup-3.0-dev libjavascriptcoregtk-4.1-dev libwebkit2gtk-4.1-dev libnm-dev
```

Compile and Install Binaries/Config-Files

```bash
$ make install
```

### Usage

View all available options with the built-in help:

```bash
$ rmenu --help
```

##### Plugins

RMenu Comes with a few default plugins.

|   Name    | Description                                             |
| :-------: | ------------------------------------------------------- |
|    run    | Execute a program in $PATH                              |
|   drun    | Run a Configured Free-Desktop Application               |
|   audio   | Select and Set-Default PulseAudio Sink using `pactl`    |
|   emoji   | Emoji picker                                            |
|   files   | Search your filesystem for matching file names          |
|  network  | Wi-Fi Login/Connection Tool using Network-Manager       |
| powermenu | Simple Power/Logout Tool (Supports Sway/Hyprland)       |
|  search   | [Bang](https://duckduckgo.com/bangs) powered search / Human calculator tool      |
|  window   | Simple Window Switcher (Supports Sway/Hyprland)         |

Run a plugin by passing the `-r` flag like one of the following:

```bash
$ rmenu -r run
$ rmenu -r drun
$ rmenu -r search
```

Or even run plugins in combination if you'd like:

```bash
$ rmenu -r run -r drun
```

##### Direct Input

Custom Menus can also be passed via `/dev/stdin` or as an input file. The schema
follows a standard as defined in [rmenu-plugin](./rmenu-plugin)

```bash
$ ./examples/rmenu-build.sh > input.json
$ rmenu -i input.json
```

When neither a plugin nor an input are specified, rmenu defaults to reading from
stdin.

```bash
$ ./examples/rmenu-build.sh | rmenu
```

##### Supported Formats

RMenu has two supported input formats: dmenu-like and JSON. JSON is the default
provided by rmenu and allows for rich configuration and controls for dynamically
generated menus. You can switch between supported formats with `-f`:

```bash
$ printf 'foo\nbar\nbaz' | rmenu -f dmenu
```

Check the [examples](./examples) folder for more examples.

### Configuration

Customize RMenu Behavior and Appearal in a
[single config](./rmenu/public/config.yaml)

Customize the entire app's appearance with CSS. A few
[Example Themes](./themes/) are available as reference. To try them out use:
`rmenu --css <my-css-theme>` or move the css file to
`$HOME/.config/rmenu/style.css`

### Scripting

RMenu plugins and imports communicate using JSON messages defined in
`rmenu-plugin`. Writing JSON in shell is painful, so rmenu provides another
cli-tool to help build messages quickly and easily while still retaining the
flexibility of JSON.

After Installing. Use the following command, and look at
[other-plugins](./other-plugins) for example uses.

```
$ rmenu-build --help
```

### Example Screenshots

#### Launchpad

![launchpad](./screenshots/launchpad.png)

#### Applet

![applet](./screenshots/applet.png)

#### Search

![search](./screenshots/search.png)

#### Nord

![nord](./screenshots/nord.png)

#### Dark

![dark](./screenshots/dark.png)

#### Solarized

![solzarized](./screenshots/solarized.png)

#### PowerMenu

![powermenu](./screenshots/powermenu.png)
