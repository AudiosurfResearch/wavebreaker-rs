-- Songs
CREATE TABLE
    songs (
        id SERIAL PRIMARY KEY,
        title TEXT NOT NULL,
        artist TEXT NOT NULL
    );