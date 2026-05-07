SELECT
	sections_fqs.uid_section,
	sections_fqs.uid_fq,
	fqs.coeff,
	sections_fqs.members_count,
	sections.title as section_title,
	fqs.title as fq_title
FROM
	sections_fqs
	INNER JOIN sections ON sections_fqs.uid_section = sections.uid
	INNER JOIN fqs ON fqs.uid = sections_fqs.uid_fq
WHERE
	sections_fqs.uid_section = ?1
ORDER BY
	fqs.position ASC