-- the types of regions we support
CREATE TYPE REGIONTYPE AS ENUM('Main', 'Marginalia');

-- A region of Text on a page (this is the bounding polygon)
CREATE TABLE region (
	id BIGINT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
	-- the page this region is on
	page BIGINT NOT NULL REFERENCES page(id),
	-- the actual bounding polygon
	polygon POLYGON NOT NULL,
	region_type REGIONTYPE NOT NULL DEFAULT 'Main'
);
