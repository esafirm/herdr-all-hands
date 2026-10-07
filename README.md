# Herdr Pluck

> Fork of [rmarganti/herdr-pluck](https://github.com/rmarganti/herdr-pluck) that adds a configurable [theme](#theme) and [browser](#browser). Plugin id: `esafirm.herdr-all-hands`.

Herdr Pluck is a Herdr plugin for quickly copying visible terminal tokens or opening visible URLs with short keyboard hints, inspired by `tmux-fingers`.

Invoke an action while a pane is focused and type the displayed hint for the item you want. The pluck action copies the selected text to your system clipboard, while the URL action opens the selection in your browser (Google Chrome by default, [configurable](#browser)). Escape or Ctrl-C cancels.

![Herdr Pluck demo](artifacts/pluck-demo-themed.gif)

## Requirements

- Herdr 0.7.4 or newer
- For release installs, a download tool:
    - `curl` or `wget`
- Rust/Cargo only when forcing a source build or when no matching prebuilt binary is available
- For copying, a system clipboard command:
    - macOS: `pbcopy`
    - Linux Wayland: `wl-copy`
    - Linux X11: `xclip` or `xsel`
- For opening URLs:
    - macOS: `open` (plus Google Chrome, or another [configured browser](#browser))
    - Linux: `google-chrome` or `xdg-open`

## Install

From the remote repository:

```bash
herdr plugin install esafirm/herdr-all-hands
```

Published releases provide prebuilt binaries for these targets:

- macOS Apple Silicon: `aarch64-apple-darwin`
- Linux x86_64: `x86_64-unknown-linux-musl`

To install a specific branch, tag, or commit, pass `--ref`:

```bash
herdr plugin install esafirm/herdr-all-hands --ref main
```

Install first downloads the GitHub Release asset matching the version in `herdr-plugin.toml`. If that asset is unavailable, it falls back to a local Cargo build when Rust is available.

From this checkout:

```bash
herdr plugin link .
```

By default, linking also installs the prebuilt binary matching `herdr-plugin.toml`. To build the checked-out source instead:

```bash
HERDR_PLUCK_BUILD_FROM_SOURCE=1 herdr plugin link .
```

Or use the Makefile, which builds from source and links in one step (`make help` lists all targets):

```bash
make install   # build ./bin/herdr-pluck and link this checkout
make build     # rebuild after code changes; the link picks it up directly
make uninstall # unlink from Herdr
```

Verify Herdr can see the action:

```bash
herdr plugin action list --plugin esafirm.herdr-all-hands
```

The action ids are:

```text
esafirm.herdr-all-hands.pluck
esafirm.herdr-all-hands.open-url
```

## Keybinding

Add a Herdr `plugin_action` binding to your Herdr config, choosing any free key you prefer:

```toml
[[keys.command]]
key = "prefix+q"
type = "plugin_action"
command = "esafirm.herdr-all-hands.pluck"
description = "pluck visible token"
```

To bind the dedicated URL action separately (`prefix+o` is Herdr's default for `open_notification_target`, so pick another key):

```toml
[[keys.command]]
key = "prefix+u"
type = "plugin_action"
command = "esafirm.herdr-all-hands.open-url"
description = "open visible URL"
```

Reload Herdr config after editing:

```bash
herdr server reload-config
```

## Usage

1. Focus a Herdr pane containing a URL, path, commit SHA, UUID, IP address, long numeric identifier, hex literal, Kubernetes reference, Git status path, branch, or diff path.
2. Invoke `esafirm.herdr-all-hands.pluck` through your keybinding or Herdr's plugin action command.
3. Herdr Pluck opens a temporary picker tab that mirrors the source layout and shows hints over copyable text in the target pane.
4. Type the shown one- or two-letter hint to copy that token and close the picker.
5. Press Escape or Ctrl-C to cancel without copying.

The `open-url` action uses the same picker flow, but shows only `http://`, `https://`, and `file://` URLs and opens the selected URL in your [browser](#browser) without changing the clipboard.

You can also invoke either action from the CLI:

```bash
herdr plugin action invoke esafirm.herdr-all-hands.pluck
herdr plugin action invoke esafirm.herdr-all-hands.open-url
```

## What gets matched

Herdr Pluck recognizes these built-in token types, in priority order:

1. URLs
2. Git status paths, Git upstream branch names, and diff paths
3. Kubernetes resource references such as `pod/nginx` or `deployment.apps/frontend`
4. File paths
5. UUIDs
6. Deployment-managed Kubernetes pod names
7. Git SHAs
8. Hex literals such as `0xdeadBEEF`
9. IPv4 addresses
10. Long numeric identifiers

Custom global patterns can be added in the plugin config directory:

```bash
CONFIG_DIR="$(herdr plugin config-dir esafirm.herdr-all-hands)"
$EDITOR "$CONFIG_DIR/config.toml"
```

Example:

```toml
[[patterns]]
name = "jira"
regex = "\\b[A-Z][A-Z0-9]+-[0-9]+\\b"
priority = 25
```

Project-local patterns are also enabled by default. Herdr Pluck looks for `.herdr-pluck.toml` from the focused pane's working directory up to the Git root. Disable or customize this in the global config:

```toml
[project]
patterns = true
pattern_files = [".herdr-pluck.toml"]
```

Project-local config files use the same `[[patterns]]` shape as global config. Pattern precedence for equal-priority overlaps is project-local, then global, then built-ins.

`regex` uses Rust regular expression syntax. If a named capture called `match` is present, only that capture is copied; otherwise the whole regex match is copied:

```toml
[[patterns]]
name = "trace-id"
regex = "trace_id=(?<match>[A-Za-z0-9_-]+)"
priority = 25
```

For `trace_id=abc123`, this pattern highlights and copies only `abc123`.

Lower `priority` values win overlapping matches. If omitted, custom pattern priority defaults to `25`.

When identical text appears more than once, every visible occurrence shows the same hint and copies the same text.

## Theme

Picker colors can be customized in the same global `config.toml`. There are three roles: `hint` (the typed label), `match` (the rest of the matched text), and `unmatched` (all other pane text). Every field is optional; unset fields keep the defaults shown here:

```toml
[theme.hint]
fg = "black"
bg = "cyan"
bold = true
dim = false

[theme.match]
fg = "yellow"
bg = "reset"

[theme.unmatched]
fg = "dark_grey"
dim = true
```

Colors accept:

- Names: `black`, `red`, `green`, `yellow`, `blue`, `magenta`, `cyan`, `white`, `grey`, and `dark_` variants (`dark_red`, `dark_grey`, ...). As in crossterm, plain names are the bright ANSI colors and `dark_` names are the normal ones; both follow your terminal palette.
- `#rrggbb` hex codes for true color.
- `0`-`255` palette indexes.
- `reset` (or `default`) for the terminal's default color.

An invalid color is reported and that field falls back to its default. Themes are only read from the global config, not project-local files.

## Browser

The `open-url` action opens URLs in Google Chrome by default. If Chrome can't be launched, it falls back to the system default handler (`open` / `xdg-open`). Change it in the global `config.toml`:

```toml
[open_url]
browser = "firefox"
```

`browser` accepts:

- An alias: `chrome`, `chromium`, `firefox`, `safari`, `arc`, `brave`, `edge`.
- Any other macOS application name (opened with `open -a <name>`), or a Linux command.
- `default` (or `system`) for the system default handler.

A browser you set explicitly doesn't fall back; if it fails, the error is shown in the picker.

For full control, set `command` to an argv list. The URL is appended as the last argument, and `command` takes precedence over `browser`:

```toml
[open_url]
command = ["open", "-na", "Google Chrome", "--args", "--profile-directory=Profile 1"]
```

Like the theme, browser settings are only read from the global config.

## Releasing binaries

Tag releases as `vX.Y.Z`. GitHub Actions validates the crate, builds release archives, and uploads platform binaries to the matching GitHub Release.

## Troubleshooting

If invoking the action does nothing useful, check that the plugin is linked and the installed binary exists:

```bash
herdr plugin link .
ls -l ./bin/herdr-pluck
herdr plugin action list --plugin esafirm.herdr-all-hands
```

If no release asset matches the plugin version, make sure Rust/Cargo is available for the local fallback build. Set `HERDR_PLUCK_BUILD_FROM_SOURCE=1` to skip the release download and build the checked-out source explicitly.

If copying fails, install one of the supported clipboard tools for your platform and try again.
