# Contributing

Thank you for your interest in contributing to Wavebreaker!
There are several ways in which you can contribute to the project:

## Non-code

You may open [issues](https://github.com/AudiosurfResearch/wavebreaker-rs/issues) to report bugs or to suggest features.

## Code

### Getting started

Make sure you [have Rust installed](https://www.rust-lang.org/tools/install). We use the nightly version. To install it using `rustup`, simply run `rustup install nightly`.
To run, this project also requires PostgreSQL (main database) and ~~Redis~~ Valkey (only has a sorted set for the global rankings for now), as well as a [Steam Web API Key](https://steamcommunity.com/dev/apikey) (used for authenticating users via Steam).
Since this project uses [Diesel](https://diesel.rs/) (an ORM for Rust), you may need to get familiar with it and its CLI for database things during development.

Clone the repository, start making changes, and when you're done, you can submit a [Pull Request](https://github.com/AudiosurfResearch/wavebreaker-rs/pulls) for review.

### What to work on?

Check the [issues](https://github.com/AudiosurfResearch/wavebreaker-rs/issues) for things that need to be worked on.
You can also check the code for things that could be improved. (Especially if there's no open issues!)
Even small improvements go a long way!

If you have a totally new idea for a feature that you'd like to try implementing, feel free to open an issue.

### Code guidelines

There aren't really any specific guidelines. Just make sure you have [Clippy](https://github.com/rust-lang/rust-clippy#usage) check your code, avoid warnings where possible and format your code using `cargo fmt`. That's it!
