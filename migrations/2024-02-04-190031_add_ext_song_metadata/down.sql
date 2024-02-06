-- This file should undo anything in `up.sql`
ALTER TABLE songs
DROP COLUMN cover_url,
DROP COLUMN cover_url_small,
DROP COLUMN mbid,
DROP COLUMN musicbrainz_title,
DROP COLUMN musicbrainz_artist,
DROP COLUMN musicbrainz_length,
DROP COLUMN mistag_lock,
DROP COLUMN aliases_artist,
DROP COLUMN aliases_title;