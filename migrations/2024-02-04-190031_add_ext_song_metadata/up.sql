CREATE TABLE
    extra_song_info (
        id SERIAL PRIMARY KEY,
        song_id INTEGER NOT NULL REFERENCES songs (id) ON DELETE CASCADE,
        cover_url TEXT,
        cover_url_small TEXT,
        mbid TEXT,
        musicbrainz_title TEXT,
        musicbrainz_artist TEXT,
        musicbrainz_length INT,
        mistag_lock BOOLEAN NOT NULL DEFAULT FALSE,
        aliases_artist TEXT[],
        aliases_title TEXT[]
    );

CREATE UNIQUE INDEX ext_info_song ON extra_song_info (song_id);