-- This file should undo anything in `up.sql`
DROP TRIGGER set_updated_at ON songs;
DROP TRIGGER set_updated_at ON extra_song_info;

ALTER TABLE songs DROP COLUMN updated_at;
ALTER TABLE extra_song_info DROP COLUMN updated_at;
