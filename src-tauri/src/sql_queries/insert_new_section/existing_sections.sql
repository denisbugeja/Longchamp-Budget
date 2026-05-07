SELECT
	uid,
	title,
	color,
	members_count,
	adults_count
FROM
	sections
WHERE
	title = ?1