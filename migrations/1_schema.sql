DROP TABLE IF EXISTS programs;

CREATE TABLE IF NOT EXISTS programs (
  id serial PRIMARY KEY,
  program_name TEXT NOT NULL UNIQUE,
  doctype TEXT NOT NULL,
  url TEXT
);
