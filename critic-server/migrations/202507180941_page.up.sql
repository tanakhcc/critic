--- A table holding pages - indexing into manuscripts
CREATE TABLE page (
	id BIGINT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
	--- the manuscript this page belongs to
	manuscript BIGINT NOT NULL REFERENCES manuscript(id),
	--- the name of this page (e.g. folio3-recto)
	name TEXT NOT NULL,
	--- the first verse on this page
	verse_start BIGINT REFERENCES verse(id),
	--- the last verse on this page
	verse_end BIGINT REFERENCES verse(id),
	--- is the minification for this image already done?
	minified BOOL NOT NULL DEFAULT false,
	--- the file extension for the image file associated with this ms
	extension TEXT NOT NULL,
	--- the pages of an individual manuscript have to have different names
	UNIQUE(manuscript, name)
);
