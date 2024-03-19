DROP TABLE IF EXISTS entries;

CREATE TABLE IF NOT EXISTS entries (
  id serial PRIMARY KEY,
  name TEXT NOT NULL,
  doctype TEXT NOT NULL,
  url TEXT,
  UNIQUE (name, doctype)
);
