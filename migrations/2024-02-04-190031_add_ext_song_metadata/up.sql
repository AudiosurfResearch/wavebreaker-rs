-- Your SQL goes here
ALTER TABLE songs
ADD COLUMN cover_url TEXT,
ADD COLUMN cover_url_small TEXT,
ADD COLUMN mbid TEXT,
ADD COLUMN musicbrainz_title TEXT,
ADD COLUMN musicbrainz_artist TEXT,
ADD COLUMN musicbrainz_length INT,
ADD COLUMN mistag_lock BOOLEAN NOT NULL DEFAULT FALSE,
ADD COLUMN aliases_artist TEXT[],
ADD COLUMN aliases_title TEXT[];