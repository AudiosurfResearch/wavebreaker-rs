# Contributing

Thank you for your interest in contributing to Wavebreaker!
Since as of now, this project is basically a small hobbyist thing that rarely sees contributions from others (people other than me, m1nt_), we are pretty relaxed about things.
Should the project grow and evolve to include regular contributors at some point, this file will be adjusted if needed. If there is something missing here that you think should be added, please let us know by opening an issue (see info on non-code contributions below).
There are several ways in which you can contribute to the project:

## Non-code

You may open [issues](https://github.com/AudiosurfResearch/wavebreaker-rs/issues) to report bugs or to suggest features.

## Code

### Getting started

Make sure you [have Rust installed](https://www.rust-lang.org/tools/install). This project aims to support the latest stable Rust version, which `rustup` should install by default.
To run, this project also requires PostgreSQL (main database) and ~~Redis~~ Valkey (used for user rankings and Steam ticket caching), as well as a [Steam Web API Key](https://steamcommunity.com/dev/apikey) (used for authenticating users via Steam). Since this project uses [Diesel](https://diesel.rs/) (an ORM for Rust), you may need to get familiar with it and its CLI for database things during development.

To get everything (including Wavebreaker's backend itself) set up in containers, you can use the included `docker-compose.yml` file to get started easily.
Alternatively, you can of course run Postgres, Valkey and the backend in any other way you see fit - just edit `Wavebreaker.toml` accordingly.

Clone the repository, start making changes, and when you're done, you can submit a [Pull Request](https://github.com/AudiosurfResearch/wavebreaker-rs/pulls) for review.

### What to work on?

Check the [issues](https://github.com/AudiosurfResearch/wavebreaker-rs/issues) for things that need to be worked on.
You can also check the code for things that could be improved. (Especially if there's no open issues!)
Even small improvements go a long way!

If you have a totally new idea for a feature or improvement, please open an issue so it's easier to coordinate and collaborate. Without them, it's harder to get help and keep track of things, since others won't know what is being worked on!

### Code guidelines

There aren't really any specific guidelines. Just make sure you have [Clippy](https://github.com/rust-lang/rust-clippy#usage) check your code, avoid warnings where possible and format your code using `cargo fmt`. That's it!
However, if you have to make some sort of decision while programming that you're unsure about, feel free to ask for advice!
