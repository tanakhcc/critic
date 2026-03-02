--- A table holding pages - indexing into manuscripts
CREATE TABLE page (
	id BIGINT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
	--- the manuscript this page belongs to
	manuscript BIGINT NOT NULL REFERENCES manuscript(id),
	--- the primary language of this page. Can be NULL to use the primary manuscript language
	language BIGINT REFERENCES language(id),
	--- the name of this page (e.g. folio3-recto)
	name TEXT NOT NULL,
	--- the first verse on this page
	verse_start BIGINT REFERENCES verse(id),
	--- the last verse on this page
	verse_end BIGINT REFERENCES verse(id),
	--- is the minification for this image already done?
	minified BOOL NOT NULL DEFAULT false,
	--- the minification has been attempted but failed
	minification_failed BOOL NOT NULL DEFAULT false,
	--- this page should be passed through kraken for baseline identification
	should_baseline BOOL NOT NULL DEFAULT true,
	--- the baseline identifiacation has been attempted but failed
	baseline_failed BOOL NOT NULL DEFAULT false,
	--- this page should be passed through kraken for ocr (identification of text based on the current baselines)
	--- (will be set when baselines have been accepted by the user)
	should_ocr BOOL NOT NULL DEFAULT false,
	--- the baseline identifiacation has been attempted but failed
	ocr_failed BOOL NOT NULL DEFAULT false,
	--- the pages of an individual manuscript have to have different names
	UNIQUE(manuscript, name)
);
