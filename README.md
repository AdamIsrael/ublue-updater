# Updater

Prototype gtk4 application to update the OS, flatpak, brew, etc, show a progress bar based on the parsed terminal output (from `uupd --json`), and possibly (stretch goal) display the changelog(s).

- re changelogs: @AdamIsrael it'd be just snagging/caching the release notes from github releases.

## TODO

- Implement the changelog display feature.
- ~~Port to libawaita~~
- ~~Use `Clamp` to limit the size of the contents. See Deja Backup for an example.~~
- ~~Rewrite the UI w/ relm4 for better async handling~~
- ~~Rewrite to use gtk-rs (again)~~
- Add support for `gettext`
- Build flatpak
- Write plugin system to update from various sources
  - uupd
  - bootc
  - rpm-ostree
  - flatpak
  - brew
  - distrobox
- Re-add terminal output once plugin(s) are enabled and we can access the output from each plugin.
