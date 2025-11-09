--- Table to hold project languages (i.e. content languges, not i18n for the frontend)
CREATE TABLE language (
	id BIGINT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
	--- the name of this language - it is the actual language code used in xml
	--- e.g. hbo-Hebr-x-Babli
	name TEXT NOT NULL UNIQUE,
	recognition_model BIGINT REFERENCES recognition_model(id),
	segmentation_model BIGINT REFERENCES segmentation_model(id)
);
