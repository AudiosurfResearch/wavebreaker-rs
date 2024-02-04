-- Your SQL goes here
DROP INDEX songs_unique_data;
CREATE UNIQUE INDEX songs_unique_data ON songs (title, artist, modifiers);