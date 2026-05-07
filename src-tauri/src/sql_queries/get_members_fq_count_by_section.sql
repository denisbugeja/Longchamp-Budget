SELECT
	COALESCE(SUM(members_count), 0) AS total
FROM
	sections_fqs
WHERE
	uid_section = ?1