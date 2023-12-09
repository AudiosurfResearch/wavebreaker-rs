CREATE EXTENSION IF NOT EXISTS "pg_trgm";

CREATE TABLE
    players (
        "id" SERIAL NOT NULL,
        "username" TEXT NOT NULL,
        "steamid64" BIGINT NOT NULL,
        "steamid32" INTEGER NOT NULL,
        "location_id" INTEGER NOT NULL DEFAULT 0,
        "account_type" INTEGER NOT NULL DEFAULT 1,
        "joined_at" TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,
        "avatar_url" TEXT NOT NULL,
        CONSTRAINT players_pk PRIMARY KEY ("id"),
        CONSTRAINT account_type_valid CHECK (account_type BETWEEN 0 AND 3),
        CONSTRAINT location_id_valid CHECK (location_id BETWEEN 0 AND 272)
    );

CREATE TABLE
    _rivalry (
        "player_a" INTEGER NOT NULL,
        "player_b" INTEGER NOT NULL,
        CONSTRAINT player_relationship_pk PRIMARY KEY ("player_id", "related_player_id"),
        CONSTRAINT player_relationship_user_fk FOREIGN KEY ("player_id") REFERENCES players ("id") ON DELETE CASCADE,
        CONSTRAINT player_relationship_related_user_fk FOREIGN KEY ("related_player_id") REFERENCES players ("id") ON DELETE CASCADE
    );