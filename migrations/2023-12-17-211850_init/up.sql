-- Players
CREATE TABLE
    players (
        id SERIAL PRIMARY KEY,
        username VARCHAR(32) NOT NULL,
        steam_id TEXT NOT NULL UNIQUE,
        steam_account_num INTEGER NOT NULL UNIQUE,
        location_id INTEGER NOT NULL DEFAULT 1,
        account_type SMALLINT NOT NULL DEFAULT 1,
        joined_at TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,
        avatar_url TEXT NOT NULL
    );

-- Rivalries
CREATE TABLE
    rivalries (
        player_id INTEGER NOT NULL REFERENCES players (id) ON DELETE CASCADE,
        rival_id INTEGER NOT NULL REFERENCES players (id) ON DELETE CASCADE,
        established_at TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,
        PRIMARY KEY (player_id, rival_id)
    );

CREATE UNIQUE INDEX rivalries_AB_unique ON rivalries (player_id, rival_id);