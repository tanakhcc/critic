--- The table that holds versification schemes
CREATE TABLE versification_scheme (
	id BIGINT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
	full_name TEXT UNIQUE NOT NULL,
	shorthand TEXT UNIQUE NOT NULL
);

-- please make sure to change the statically used schemes throughout the code if you ever change this or add more static schemes in later migrations
INSERT INTO versification_scheme (full_name, shorthand) VALUES ('Common', 'C'), ('Present', 'P');
