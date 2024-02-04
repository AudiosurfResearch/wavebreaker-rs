-- This file should undo anything in `up.sql`
DROP INDEX songs_unique_data;
CREATE UNIQUE INDEX songs_unique_data ON songs (title, artist);