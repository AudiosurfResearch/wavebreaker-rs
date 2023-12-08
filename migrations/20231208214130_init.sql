CREATE EXTENSION IF NOT EXISTS "pg_trgm";
CREATE TABLE users (
    "id" SERIAL NOT NULL,
    "username" TEXT NOT NULL,
    "steamid64" BIGINT NOT NULL,
    "steamid32" INTEGER NOT NULL,
    "location_id" INTEGER NOT NULL DEFAULT 0,
    "account_type" INTEGER NOT NULL DEFAULT 1,
    "joined_at" TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "avatar_url" TEXT NOT NULL,
    CONSTRAINT users_pk PRIMARY KEY ("id"),
    CONSTRAINT account_type_valid CHECK(
        account_type between 0 and 3
    ),
    CONSTRAINT location_id_valid CHECK(
        location_id between 0 and 272
    )
);