# wavebreaker-rs
**EXTREMELY WIP!!!** (Yes, so much so that it's worth three exclamation marks!!!)

Config example:
```toml
[main]
address = "localhost:1337"
database = "postgres://user:owaranai_future_sound_o@localhost/wavebreaker"

[external]
steam_key = "music_bokura_zutto_so_hype"
```

To connect, use the latest Wavebreaker client with ``forceInsecure`` set to ``true`` in its config. This is only intended for local testing.
*The end goal of this endeavour is to have feature parity with the [TypeScript version!](https://github.com/AudiosurfResearch/Wavebreaker)*

## What works currently?
- Logging in/registering via Steam
- Leaderboards
- Automatically adding Steam friends as rivals
- Creating and retrieving songs from the DB
- Submitting scores
- **Audiosurf Radio!**
- **Proper dethrones and the Brutus achievement!** (not present in TypeScript version)

## What still needs to be done?
- MusicBrainz integration
- Non-game API (for the frontend)
- Miscellaneous things (shouts, custom news, etc.)

## Contributing

*See [CONTRIBUTING.md](https://github.com/AudiosurfResearch/wavebreaker-rs/blob/master/CONTRIBUTING.md).*
