CREATE TABLE accounts (
	id INTEGER PRIMARY KEY AUTOINCREMENT,
	email VARCHAR NOT NULL UNIQUE,
	firstname VARCHAR,
	lastname VARCHAR NOT NULL,
	password VARCHAR NOT NULL
);
