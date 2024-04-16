DROP TABLE IF EXISTS entries;

CREATE TABLE IF NOT EXISTS entries (
  id serial PRIMARY KEY,
  name TEXT NOT NULL,
  UNIQUE (name)
);
