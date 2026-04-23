# Contributing

LACT is entirely written in Rust, but depends on several C libraries. See [README](../README.md#building-from-source) for the list.

For development builds, it is enough to simply run `cargo build`, `make` is not necessary. 

Often times you might want to test local daemon changes togther with the UI. To do so, you can do the following:
- Stop the system daemon (if installed)
- `cargo build && sudo ./target/debug/lact daemon` in one terminal
- `cargo run` in another terminal to run the GUI

# Hardware

LACT supports multiple GPU vendors, which often have very different behaviour. This affects both the daemon and parts of the GUI.

When making changes, please make as few assumptions as possible about the hardware and what wil be available, as these assumptions may not hold true in all cases, even if they seem logical.

One example of this is that certain GPU generations only report fan speed as an absolute RPM value, while others only report it as a percentage value, and most report both. The code needs to be able to handle such edge cases gracefully.

It is possible to run the GUI pointed to a device snapshot. This only works for simulating AMD cards. With the daemon stopped, you can run something like this:
```
_LACT_DRM_SYSFS_PATH=./lact-daemon/src/tests/data/amd/rx9070 cargo run
```
This will display the UI mostly the way it looks on that device (though some things will be missing). It is also readonly, applying settings is not supported.

# Tests

For running the tests, you can simply use `cargo test`.

Certain tests may use snapshot testing via [cargo insta](https://insta.rs/docs/cli/). 
You do not need to install anything extra to run them, but the `cargo insta` CLI is useful for updating test snapshots (mainly used in the daemon for device-specific info and commands).

If you make any changes to what device info is reported or how settings are applied, you can run
```
cargo insta test
cargo insta review
```
And then use an interactive UI to review if the changes are desirable or not.

# Dependencies

It is generally preferable to avoid adding new dependencies as much as possible.

If the `Cargo.lock` file changes in any way, you will also need to update the `./flatpak/generated-sources.json` file, otherwise the flatpak build will fail.

You can regenerate it with [this script](https://github.com/flatpak/flatpak-builder-tools/blob/master/cargo/flatpak-cargo-generator.py) from flatpak-builder-tools:
```
flatpak-cargo-generator.py -o flatpak/generated-sources.json Cargo.lock
```
