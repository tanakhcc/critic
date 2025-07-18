--- The table that holds information on transcriptions
--- a transcription is basically a blob of TEI/XML data for an individual page of a manuscript
CREATE TABLE transcription (
	id BIGINT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
	--- the page this transcription is for (also includes information about the manuscript)
	page BIGINT REFERENCES page(id),
	--- does the user wish this transcription to be viewable
	published BOOL NOT NULL DEFAULT false,
	--- the name of the user who created this transcription
	username TEXT REFERENCES user_session(username),
	--- only one version for a page and a user is saved
	UNIQUE(page, username)
	--- the actual xml data resides on disk at `data_directory`/transcript/manuscript/page/user.tei.xml
);
