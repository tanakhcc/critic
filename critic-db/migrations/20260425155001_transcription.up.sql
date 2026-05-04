--- The table that holds information on transcriptions
--- a transcription is basically a blob of TEI/XML data for an individual page of a manuscript
CREATE TABLE transcription (
	id BIGINT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
	--- the baseline this transcription is for
	line BIGINT NOT NULL REFERENCES line(id),
	--- does the user wish this transcription to be viewable
	published BOOL NOT NULL DEFAULT false,
	--- the name of the user who created this transcription
	username TEXT NOT NULL REFERENCES user_session(username),
	--- the actual xml data parsable with the `page_from_xml` function
	content TEXT NOT NULL,
	--- only one version for a page and a user is saved
	UNIQUE(line, username)
);
