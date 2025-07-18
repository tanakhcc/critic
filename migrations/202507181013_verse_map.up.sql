--- table containing mapping from non-semantic verse-ids to verse-numbers in different versification schemes
CREATE TABLE verse_map (
	--- the (non-semantic) verse id mapped to a semantic verse-number
	verse_id BIGINT NOT NULL REFERENCES verse(id),
	--- the versification scheme for this map
	versification_scheme BIGINT NOT NULL REFERENCES versification_scheme(id),
	--- the verse number in the scheme, e.g. Gen 5:17
	verse_nr TEXT NOT NULL,
	--- each verse may only be mapped to one verse number in each scheme
	UNIQUE(verse_id, versification_scheme)
);
