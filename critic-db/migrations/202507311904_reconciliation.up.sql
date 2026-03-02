--- 
CREATE TABLE reconciliation (
	id BIGINT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
	--- the page this transcription is for
	--- each page may only be reconciled once
	page BIGINT UNIQUE NOT NULL REFERENCES page(id),
	--- the name of the user who created this reconciliation
	username TEXT NOT NULL REFERENCES user_session(username)
);
