# Contributing

## AI Policy

The most important rule is that **all communication must be human-written**.
Do not make others to talk to a bot, or force them to understand blocks of LLM-generated text.

### Reporting issues or feature requests
- You must describe the problem/feature in your own words.
- If you have used an AI tool to analyze or debug an issue, you must distinguish parts which have been verified by you vs what AI analysis told you.
- Rules for reporting using AI:
  - After the summary, include this statement: "Reported with AI assistance following ruleset v1."
  - AI investigation and analysis should be hidden under spoiler if present
  - Use Simplified Technical English
  - Be practical and focused with the context provided, only relevant information
  - Your goal is to describe the problem and means to reproduce, not to solve it
  - Usually, the most important piece of information for daemon-related problems is the snapshot. Attach it with the user's consent.


### Contributing code
- You must understand the code you are submitting. If asked about a specific change, you need to be able to explain why you did it.
- Avoid needlessly verbose comments, checks against impossible cases, or other things which make the code more complex than it needs to be.
- Do not submit unnecessarily large PRs which are difficult to review.
- It is better to leave a well-formulated feature request, than a purely vibecoded implementation.
  - De-slopping someone else's code as a maintainer often takes more time than implementing the same thing from scratch.
- Create an issue for discussion before submitting a PR for a large feature.
  - Features that add a lot of complexity, but only benefit a small number of users (e.g. something that only works on select GPU models) may not be worth maintaining.
- If you receive feedback on a PR, you must think about it yourself, not just submit it as the next prompt.

## Overview

LACT is entirely written in Rust, but depends on several C libraries. See [README](../README.md#building-from-source) for the list.

For development builds, it is enough to simply run `cargo build`, `make` is not necessary. 

Often times you might want to test local daemon changes togther with the UI. To do so, you can do the following:
- Stop the system daemon (if installed)
- `cargo build && sudo ./target/debug/lact daemon` in one terminal
- `cargo run` in another terminal to run the GUI

## Hardware

LACT supports multiple GPU vendors, which often have very different behaviour. This affects both the daemon and parts of the GUI.

When making changes, please make as few assumptions as possible about the hardware and what wil be available, as these assumptions may not hold true in all cases, even if they seem logical.

One example of this is that certain GPU generations only report fan speed as an absolute RPM value, while others only report it as a percentage value, and most report both. The code needs to be able to handle such edge cases gracefully.

It is possible to run the GUI pointed to a device snapshot. This only works for simulating AMD cards. With the daemon stopped, you can run something like this:
```
_LACT_DRM_SYSFS_PATH=./lact-daemon/src/tests/data/amd/rx9070 cargo run
```
This will display the UI mostly the way it looks on that device (though some things will be missing). It is also readonly, applying settings is not supported.

## Tests

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
