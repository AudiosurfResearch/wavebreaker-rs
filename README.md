# wavebreaker-rs
**EXTREMELY WIP!!!** (Yes, so much so that it's worth three exclamation marks!!!)

This is a custom server for Dylan Fitterer's game [Audiosurf](https://store.steampowered.com/app/12900/AudioSurf/).

*The goal of this endeavour is to have at least feature parity with the [TypeScript version](https://github.com/AudiosurfResearch/Wavebreaker), which is currently running in production at https://wavebreaker.arcadian.garden. Once this goal is reached and everything is working after a public test, the TypeScript version will be replaced with this.*

Config example (``Wavebreaker.toml``):
```toml
[main]
address = "localhost:1337"
database = "postgres://user:owaranai_future_sound_o@localhost/wavebreaker"
redis = "redis://localhost:6379"
jwt_secret = "kono oto o kiku subete ga 「　　　」"

[radio]
cgr_location = "./radio"

[external]
steam_key = "music_bokura_zutto_so_hype"
steam_realm = "http://localhost:1337"
steam_return_path = "/api/auth/return"
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

## What works currently?
- Logging in/registering via Steam
- Leaderboards
- Automatically adding Steam friends as rivals
- Creating and retrieving songs from the DB
- Submitting scores
- Shouts (Song comments)
- Player rankings (using Skill Points)
- **Audiosurf Radio!** (using [Rainbow Dream](https://github.com/AudiosurfResearch/rainbowdream) to create the needed files)
- **MusicBrainz integration!** (fetches cover art and title)
- **Proper dethrones and the Brutus achievement!** *(not present in TypeScript version)*
- **Custom aliases for songs' title/artist names** *(not present in TypeScript version)*
- Miscellaneous stuff (e.g. custom news)

## What still needs to be done?
- Non-game API (for the frontend and other clients that want to get data from Wavebreaker)

## Contributing

*See [CONTRIBUTING.md](https://github.com/AudiosurfResearch/wavebreaker-rs/blob/master/CONTRIBUTING.md).*
