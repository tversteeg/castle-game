# [castle-game](https://tversteeg.itch.io/castle-game)
A free & open source 2D Lemmings-meets-tower defense destructible terrain game.

[![](https://travis-ci.org/tversteeg/castle-game.svg?branch=master)](https://travis-ci.org/tversteeg/castle-game) 
[![](https://img.shields.io/crates/d/castle-game.svg)](#downloads)
[![](https://img.shields.io/crates/v/castle-game.svg)](https://crates.io/crates/castle-game)
[![](https://img.shields.io/github/commits-since/tversteeg/castle-game/latest.svg)]()

![Screenshot](https://github.com/tversteeg/castle-game-assets/blob/master/screengrab.gif?raw=true)

# Run

## From Github

You can download the [latest](https://github.com/tversteeg/castle-game/releases/latest) version from [releases](https://github.com/tversteeg/castle-game/releases). It should work by just running the executable, if not, [create an issue](https://github.com/tversteeg/castle-game/issues/new).

## Rust

Or if you have Rust installed you can do the following:

```bash
cargo install --force castle-game
castle-game
```

# Building

## Prerequisites

To build the project you need to have [Rust](https://www.rustup.rs/) installed.

### Linux (Debian based)

    sudo apt install xorg-dev cmake

### Windows & Mac

You need to install [CMake](https://cmake.org/) and make sure it's in your path.

## Run

Check out the repository with git and build:

    git clone https://github.com/tversteeg/castle-game && cd castle-game
    
Build & run:
    
    cargo run --release

# Contributing

Contributions are more than welcome!
