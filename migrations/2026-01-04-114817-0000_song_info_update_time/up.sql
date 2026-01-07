-- Your SQL goes here
ALTER TABLE songs ADD updated_at TIMESTAMPTZ(3) NOT NULL DEFAULT CURRENT_TIMESTAMP;
ALTER TABLE extra_song_info ADD updated_at TIMESTAMPTZ(3) NOT NULL DEFAULT CURRENT_TIMESTAMP;

SELECT diesel_manage_updated_at('songs');
SELECT diesel_manage_updated_at('extra_song_info');
