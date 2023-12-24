-- Players
CREATE TABLE
    players (
        id SERIAL PRIMARY KEY,
        username VARCHAR(32) NOT NULL,
        steam_id BIGINT NOT NULL UNIQUE,
        location_id INTEGER NOT NULL DEFAULT 1,
        account_type SMALLINT NOT NULL DEFAULT 1,
        joined_at TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,
        avatar_url TEXT NOT NULL
    );

CREATE UNIQUE INDEX player_steam_id_key ON players (steam_id);

-- Rivalries
CREATE TABLE
    rivalries (
        player INTEGER NOT NULL,
        rival INTEGER NOT NULL,
        established_at TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP
    );

ALTER TABLE rivalries
ADD CONSTRAINT rivalries_A_fkey FOREIGN KEY (player) REFERENCES players (id) ON DELETE CASCADE ON UPDATE CASCADE;

ALTER TABLE rivalries
ADD CONSTRAINT rivalries_B_fkey FOREIGN KEY (rival) REFERENCES players (id) ON DELETE CASCADE ON UPDATE CASCADE;

CREATE UNIQUE INDEX rivalries_AB_unique ON rivalries ("A", "B");