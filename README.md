# rlbot-rs

## CLI

### Install

`cargo install --git https://github.com/swz-git/rlbot-match-manager`

### Usage

`rlbot-mr --help`

## Flatbuffers

Use this command to generate `lib/src/flat_wrapper/rlbot_generated.rs` from `lib/rlbot.fbs`:

`cd lib && flatc -r --gen-object-api --object-suffix "Object" -o src/flat_wrapper rlbot.fbs`

You also need to remove the lines saying `#[non_exhaustive]` as otherwise we can't export the flatbuffers types.