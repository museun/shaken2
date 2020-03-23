-- drop the table because we'll require them to build it each start
DROP TABLE IF EXISTS builtin_commands;
-- and then create it
CREATE TABLE builtin_commands (
    id   INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT UNIQUE,
    name TEXT NOT NULL,
    UNIQUE(name)
);
-- and ensure the user_commands table exists
CREATE TABLE IF NOT EXISTS user_commands (
    id         INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT UNIQUE,
    name       TEXT NOT NULL,
    body       TEXT NOT NULL,
    room       INTEGER NOT NULL,
    uses       INTEGER NOT NULL,
    owner      TEXT NOT NULL,     -- sqlx can't do i64s apparently
    disabled   BOOLEAN NOT NULL,  -- but it can do booleans
    created_at BLOB NOT NULL,
    UNIQUE(room, name)
)
