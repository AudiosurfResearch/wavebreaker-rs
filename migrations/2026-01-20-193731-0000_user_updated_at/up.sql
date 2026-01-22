-- Your SQL goes here
ALTER TABLE players ADD updated_at TIMESTAMPTZ(3) NOT NULL DEFAULT CURRENT_TIMESTAMP;

SELECT diesel_manage_updated_at('players');
