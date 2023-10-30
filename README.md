# rlbot-match-manager

A library and cli tool that can start matches with rlbot

## CLI

### Install

`cargo install --git https://github.com/swz-git/rlbot-match-manager`

### Usage

`rlbot-mm --help`

## Flatbuffers

Use this command to generate `lib/src/rlbot_generated.rs` from `lib/rlbot.fbs`:

`cd lib && flatc -r -o src rlbot.fbs`
