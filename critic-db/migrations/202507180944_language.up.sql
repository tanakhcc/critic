--- Textdirection for this Language, passed to kraken
CREATE TYPE TextDirection AS ENUM ('HorizontalLR', 'HorizontalRL', 'VerticalLR', 'VerticalRL');

--- Table to hold project languages (i.e. content languges, not i18n for the frontend)
CREATE TABLE language (
	id BIGINT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
	--- the name of this language - it is the actual language code used in xml
	--- e.g. hbo-Hebr-x-Babli
	name TEXT NOT NULL UNIQUE,
	--- the model used for character recognition
	recognition_model BIGINT REFERENCES recognition_model(id),
	--- the model used for baseline segmentation
	segmentation_model BIGINT REFERENCES segmentation_model(id),
	--- Only characters in this alphabet are considered when we try to find out whether two
	--- texts are equal
	equality_alphabet TEXT,
	--- text direction for this language, used in kraken
	text_direction TextDirection NOT NULL DEFAULT 'HorizontalLR'
);
