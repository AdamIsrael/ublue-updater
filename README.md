# Updater

Prototype gtk4 application to update the OS, flatpak, brew, etc, show a progress bar based on the parsed terminal output (from `uupd --json`), and possibly (stretch goal) display the changelog(s).

- re changelogs: @AdamIsrael it'd be just snagging/caching the release notes from github releases.

## TODO

- Implement the changelog display feature.
- ~~Port to libawaita~~
- ~~Use `Clamp` to limit the size of the contents. See Deja Backup for an example.~~

### uupd

When `uupd` emits the json, it's at the beginning of the update step rather than when the update is complete. That means the progress bar will be at 100% when the last step is still running, and resets when the update is actually complete.

This behaviour could be updated in `uupd` itself, or we can add logic to work with the current implementation.
