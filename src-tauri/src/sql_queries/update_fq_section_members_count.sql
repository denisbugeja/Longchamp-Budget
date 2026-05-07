UPDATE
	sections_fqs
SET
	members_count = ?1
WHERE
	uid_section = ?2
	AND uid_fq = ?3