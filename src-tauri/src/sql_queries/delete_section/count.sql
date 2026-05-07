SELECT
	COUNT(uid)
FROM
	expenses_instances
WHERE
	uid_section = ?1