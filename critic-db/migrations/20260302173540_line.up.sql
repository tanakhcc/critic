--- OCR results for one line
--- this table contains one baseline with its associated OCR Index
CREATE TABLE line (
	id BIGINT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
	--- the region this line belongs to
	region BIGINT NOT NULL REFERENCES region(id),
	--- the baseline. coordinates are +x to the right, +y to the bottom, starting from the top-left, in px
	baseline LSEG NOT NULL,
	--- the basetext proposed after running OCR and finding the correct spot in the base corpus
	--- Before OCR is run, this will be NULL, since we only know the baseline itself.
	---
	--- this is critic-tei-xml
	proposed_basetext TEXT
);

