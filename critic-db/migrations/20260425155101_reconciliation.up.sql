--- 
CREATE TABLE reconciliation (
	id BIGINT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
	--- the page this transcription is for
	--- each page may only be reconciled once
	page BIGINT UNIQUE NOT NULL REFERENCES page(id),
	--- the name of the user who created this reconciliation
	username TEXT NOT NULL REFERENCES user_session(username),
	--- the actual content in this reconciliation - this is a blob of TEI XML
	--- parsable with the page_from_xml function
	content TEXT NOT NULL
);
