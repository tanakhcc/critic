--- The table that holds versification schemes
CREATE TABLE versification_scheme (
	id INT NOT NULL GENERATED ALWAYS AS IDENTITY,
	full_name TEXT UNIQUE NOT NULL,
	shorthand TEXT UNIQUE NOT NULL
);

INSERT INTO versification_scheme (full_name, shorthand) VALUES ('Common', 'C'), ('Present', 'P');
