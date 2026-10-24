# scramble-ui

Material Design 3 Expressive for [iced](https://iced.rs): color schemes
from a seed, type scale, icons, shapes, spring motion, buttons, toolbars,
side sheets, popovers, drop-downs, fields and resize handles, with the
desktop's accent color and light or dark preference (Omarchy, the XDG settings portal,
Windows and macOS). Shared by [prev](https://github.com/scrambletools/prev),
[triib](https://github.com/scrambletoolsllc/triib) and
[plaa](https://github.com/scrambletools/plaa).

## Using it

Cargo applies `[patch.crates-io]` only in the top level workspace, so an
app using this crate adds the same patch section as this crate's
`Cargo.toml`, pointing at `vendor/` (see `vendor/PATCHES.md`):

```toml
[patch.crates-io]
iced_graphics = { path = "../scramble-ui/vendor/iced_graphics" }
iced_widget = { path = "../scramble-ui/vendor/iced_widget" }
winit = { path = "../scramble-ui/vendor/winit" }
```

Load the fonts at startup with `scramble_ui::font::files()`, and set the
components' own labels in the app's language with
`scramble_ui::labels::set`.

## Patches, and how they go away

prev, triib and plaa patch `iced_graphics`, `iced_widget` and `winit`
with the copies in `vendor/` (prev keeps its own winit copy, and plaa and
prev also patch `smithay-clipboard`). Cargo applies `[patch]` only in the
program being built, never for a library's users, so this crate cannot go
to crates.io while it needs them, and a distribution that builds from its
own copy of each crate, as Debian and Fedora do, would build the apps on
stock iced and winit. The aim: this crate needs no patch, an app patches
only what cannot be done another way without loss, and none of the apps
loses anything on any platform.

1. **This crate takes over the iced patches** as widgets of its own:
   the text input, the pick list and its menu, the tooltip and the
   scrollable, with our changes. It finds cursor positions in right to
   left text from the paragraph's own layout (`Paragraph::buffer`) and
   gives canvas text a finite width. It reads the keyboard layout on
   Linux from a keyboard of its own on the app's Wayland connection, as
   iced's clipboard adds a data device there, or from XKB on X11, and
   turns iced's hand cursor into the arrow. The apps then drop the
   `iced_graphics` and `iced_widget` patches, and this crate is
   published on crates.io.
2. **The apps keep the winit patch** for what only winit can do, all of
   it on macOS: winit's default menu giving way to triib's menu bar, Cut,
   Copy, Paste and Select All reaching text fields from it, windows
   following the system's light or dark appearance, a new cursor shown
   without moving the mouse, and for prev, files from Finder and the Dock
   and drags of any type. A Linux distribution can drop the patch and
   lose nothing.
3. **plaa and prev keep their smithay-clipboard patch**, for drag and
   drop on Wayland and one clipboard worker shared by all windows.
4. **Not done:** replacing winit's methods at run time to get prev's
   files and drags without the patch. That breaks quietly when winit
   changes; a patch that no longer applies fails the build instead.

Each patch is offered upstream, small and one change at a time, and
shrinks as it lands:

| When | Then |
|---|---|
| winit's 0.30 branch takes our macOS changes | The apps drop them from the patch; triib and plaa drop the winit patch once nothing they use is left in it |
| iced takes the scrollable fix or right to left text input | This crate drops its copy of that widget for iced's |
| iced releases 0.15 | This crate brings its widgets up to it, as the patches are applied again today, and the winit patch moves to the winit iced then uses. iced's own winit fork (`iced-rs/winit`: macOS URL events, Wayland key repeat and fine wheel scrolling, a newer sctk) reaches only iced's development branch; whether 0.15's published crates get it decides what the patch starts from |
| iced moves to winit 0.31 | prev's files need no patch, as 0.31 leaves the application delegate to the app, nor its drags, through 0.31's drag and drop API (rust-windowing/winit#4571); the macOS changes are checked against 0.31 and offered there |
| winit gives the keyboard layout (rust-windowing/winit#2678) | This crate reads it from winit again |
| smithay-clipboard takes prev's changes | plaa and prev drop that patch |

## Assistant models

`scramble_ui::assistant` keeps the assistant's models in a file the apps
share, `scramble/assistant.toml` in the config folder, so a model set up
in one app is there in the others, and `changes` tells a running app
when another saved it. The keys are in the system keychain, under
`keychain_service`.

## Icons

`src/icon.rs` lists every icon any app draws. After adding one, rebuild
the subset fonts:

```
uv run --with fonttools scripts/build-fonts.py
```

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or
[MIT license](LICENSE-MIT) at your option. The bundled fonts keep their own
licenses: Roboto Flex under the SIL Open Font License 1.1
(`assets/fonts/OFL.txt`), Material Symbols under Apache 2.0
(`assets/fonts/LICENSE-MaterialSymbols.txt`). The vendored iced crates are
MIT (`vendor/*/LICENSE`).
