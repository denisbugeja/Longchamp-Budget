SELECT
	sections_fqs.uid_section,
	COALESCE(SUM(members_count), 0) AS total
FROM
	sections_fqs
GROUP BY
	sections_fqs.uid_section