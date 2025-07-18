--- Table containing manuscripts
CREATE TABLE manuscript (
	id BIGINT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
	--- title of the manuscript (e.g. ML115)
	title TEXT NOT NULL UNIQUE,
	--- holding institution if any (e.g. Saint Petersburg, RNL)
	institution TEXT,
	--- collection this manuscript belongs to (e.g. Firkovitch EBR II)
	collection TEXT,
	--- description of the different hands used in this manuscript
	hand_desc TEXT,
	--- description of the script in this manuscript
	script_desc TEXT
);
