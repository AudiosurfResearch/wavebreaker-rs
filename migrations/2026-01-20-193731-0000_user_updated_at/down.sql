-- This file should undo anything in `up.sql`
DROP TRIGGER set_updated_at ON players;

ALTER TABLE players DROP COLUMN updated_at;
