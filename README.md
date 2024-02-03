# wavebreaker-rs
**EXTREMELY WIP!!!** (Yes, so much so that it's worth three exclamation marks!!!)

Config example (``Wavebreaker.toml``):
```toml
[main]
address = "localhost:1337"
database = "postgres://user:owaranai_future_sound_o@localhost/wavebreaker"

[radio]
cgr_location = "./radio"

[external]
steam_key = "music_bokura_zutto_so_hype"
```

Radio song list example (``WavebreakerRadio.toml``):
```toml
[[radio_songs]]
id = 1 # ID of the song on the server (song has to be known to the server already!)
title = "Dear Music." # Don't use non-ASCII characters
artist = "A4." # here too!
external_url = "https://www.youtube.com/watch?v=XeVrdjZSceA" # Put a link to buy (not stream!) the song here, if possible!
cgr_url = "http://localhost/as/asradio/WVBR_A4_DearMusic.cgr" # URL for the .cgr file containing the song,
```

To connect, use the latest Wavebreaker client with ``forceInsecure`` set to ``true`` in its config. This is only intended for local testing.
*The end goal of this endeavour is to have feature parity with the [TypeScript version!](https://github.com/AudiosurfResearch/Wavebreaker)*

## What works currently?
- Logging in/registering via Steam
- Leaderboards
- Automatically adding Steam friends as rivals
- Creating and retrieving songs from the DB
- Submitting scores
- Shouts (Song comments)
- **Audiosurf Radio!**
- **Proper dethrones and the Brutus achievement!** (not present in TypeScript version)
- Miscellaneous stuff (e.g. custom news)

## What still needs to be done?
- User rankings
- MusicBrainz integration
- Non-game API (for the frontend)

## Contributing

*See [CONTRIBUTING.md](https://github.com/AudiosurfResearch/wavebreaker-rs/blob/master/CONTRIBUTING.md).*
