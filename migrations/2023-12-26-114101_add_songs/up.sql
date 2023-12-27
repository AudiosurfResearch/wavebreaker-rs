-- Songs
CREATE TABLE
    songs (
        id SERIAL PRIMARY KEY,
        title TEXT NOT NULL,
        artist TEXT NOT NULL,
        created_at TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP
    );

CREATE UNIQUE INDEX songs_unique_data ON songs (title, artist);