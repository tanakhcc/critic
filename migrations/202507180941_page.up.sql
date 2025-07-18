--- A table holding pages - indexing into manuscripts
CREATE TABLE page (
	id BIGINT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
	--- the manuscript this page belongs to
	manuscript BIGINT REFERENCES manuscript(id),
	--- the name of this page (e.g. folio3-recto)
	name TEXT NOT NULL,
	--- the first verse on this page
	verse_start BIGINT REFERENCES verse(id),
	--- the last verse on this page
	verse_end BIGINT REFERENCES verse(id),
	--- the pages of an individual manuscript have to have different names
	UNIQUE(manuscript, name)
);
