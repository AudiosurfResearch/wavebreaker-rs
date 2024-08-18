-- Your SQL goes here
SET timezone = 'UTC';

ALTER TABLE players ALTER joined_at TYPE timestamptz(3), ALTER joined_at SET DEFAULT now();
ALTER TABLE scores ALTER submitted_at TYPE timestamptz(3), ALTER submitted_at SET DEFAULT now();
ALTER TABLE songs ALTER created_at TYPE timestamptz(3), ALTER created_at SET DEFAULT now();
ALTER TABLE rivalries ALTER established_at TYPE timestamptz(3), ALTER established_at SET DEFAULT now();
ALTER TABLE shouts ALTER posted_at TYPE timestamptz(3), ALTER posted_at SET DEFAULT now();