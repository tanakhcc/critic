--- Table holding at most one row, containing the default primary language used for manuscripts
CREATE TABLE default_language (
	--- force only one id to ever exist - this table has one row
	id BIGINT PRIMARY KEY GENERATED ALWAYS AS (1) STORED UNIQUE,
	language BIGINT REFERENCES language(id)
);

INSERT INTO default_language (language) VALUES (NULL);
