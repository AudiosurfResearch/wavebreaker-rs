# wavebreaker-rs
**EXTREMELY WIP!!!!!** (Yes, so much so that it's worth five exclamation marks!!!!!)

Config example:
```toml
[main]
address = "localhost:1337"
database = "postgres://user:owaranai_future_sound_o@localhost/wavebreaker"

[external]
steam_key = "music_bokura_zutto_so_hype"
```

To connect, use the latest Wavebreaker client with ``forceInsecure`` set to ``true`` in its config. This is only intended for local testing.
*The end goal of this endeavour is to have feature parity with the TypeScript version!*

## What works currently?
- Logging in/registering via Steam
- Automatically adding Steam friends as rivals
- Creating and retrieving songs from the DB