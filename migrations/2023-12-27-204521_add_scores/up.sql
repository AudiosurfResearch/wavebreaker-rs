-- Scores
CREATE TABLE
    scores (
        -- Basic info
        id SERIAL PRIMARY KEY,
        song_id INTEGER NOT NULL REFERENCES songs (id) ON DELETE CASCADE,
        player_id INTEGER NOT NULL REFERENCES players (id) ON DELETE CASCADE,
        league INTEGER NOT NULL,
        submitted_at TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,
        play_count INTEGER NOT NULL DEFAULT 0,
        -- Here we go
        score INTEGER NOT NULL,
        track_shape INTEGER[256] NOT NULL,
        xstats INTEGER[] NOT NULL,
        density INTEGER NOT NULL,
        vehicle INTEGER NOT NULL,
        feats TEXT[],
        song_length INTEGER NOT NULL,
        gold_threshold INTEGER NOT NULL,
        iss INTEGER NOT NULL,
        isj INTEGER NOT NULL,
    );

CREATE UNIQUE INDEX scores_unique_compound ON scores (user_id, song_id, league);