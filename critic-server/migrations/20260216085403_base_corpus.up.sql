--- base corpus
--- this table contains metadata about the base elements of the corpus known
--- the base corpus is made up of blocks (short units of a few sentences)
--- these may be in different languages and are any valid critic-tei-xml
--- primarily, these blocks will contain clear text (the textual base of your corpus)
--- with anchors.
--- during ocr, the OCRed text in a line will be Full-Text-Searched in the base corpus
--- note that a cleansed version (just the surface text, and only units specified in the languages equality alphabet)
CREATE TABLE base_corpus (
	id BIGINT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
	--- the language of this block
	language BIGINT REFERENCES language(id),
	--- this is the critic-tei-xml of this block, which should deserialize to `Vec<streamed::InlineBlock>`
	content TEXT NOT NULL,
	--- versification scheme used in this block
	versification_scheme BIGINT REFERENCES versification_scheme(id),
	--- first verse in this block
	verse_start BIGINT REFERENCES verse(id),
	--- last verse in this block
	verse_end BIGINT REFERENCES verse(id)
);
