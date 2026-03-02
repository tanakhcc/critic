--- Table holding an index of all verses
CREATE TABLE verse (
	--- this is a non-semantic ID of this verse
	id BIGINT PRIMARY KEY GENERATED ALWAYS AS IDENTITY
);
