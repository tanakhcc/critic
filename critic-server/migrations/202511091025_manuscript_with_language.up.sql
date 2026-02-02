--- Select manuscript with language as Option<String>
CREATE VIEW manuscript_with_language AS
SELECT
    id,
    title,
    institution,
    collection,
	hand_desc,
	script_desc,
	COALESCE(
		manuscript.language,
		(SELECT language FROM default_language
		 WHERE id = 1)) as language,
	(SELECT name FROM language
	 WHERE id = COALESCE(
		manuscript.language,
		(SELECT language FROM default_language
		 WHERE id = 1))) as language_name
	FROM manuscript;

