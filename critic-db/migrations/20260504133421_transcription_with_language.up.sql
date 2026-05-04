--- Select transcription with its default language as Option<String>
CREATE VIEW transcription_with_language AS
SELECT
    transcription.id,
    line.id as line,
	region.id as region,
	page.id as page,
	manuscript.id as manuscript,
    content,
    published,
	username,
	COALESCE(
		manuscript.language,
		page.language,
		(SELECT language FROM default_language
		 WHERE id = 1)) as language,
	(SELECT name FROM language
	 WHERE id = COALESCE(
		manuscript.language,
		page.language,
		(SELECT language FROM default_language
		 WHERE id = 1))) as language_name
	FROM transcription
	INNER JOIN line on transcription.line = line.id
	INNER JOIN region on line.region = region.id
	INNER JOIN page on region.page = page.id
	INNER JOIN manuscript on page.manuscript = manuscript.id;

