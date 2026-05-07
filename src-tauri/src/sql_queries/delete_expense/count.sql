SELECT
	COUNT(uid)
FROM
	expenses_instances
WHERE
	uid_expense = ?1