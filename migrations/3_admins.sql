DROP TABLE IF EXISTS admins;

CREATE TABLE IF NOT EXISTS admins (
  id serial PRIMARY KEY,
  username TEXT,
  token TEXT NOT NULL UNIQUE
);
