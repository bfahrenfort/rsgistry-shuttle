DROP TABLE IF EXISTS queue;

CREATE TABLE IF NOT EXISTS queue (
  id serial PRIMARY KEY,
  program_name TEXT NOT NULL,
  doctype TEXT NOT NULL,
  url TEXT,
  request_type TEXT NOT NULL
);
