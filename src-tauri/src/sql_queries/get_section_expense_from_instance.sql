SELECT
	expenses_instances.uid_section,
	expenses_instances.uid_expense,
	sections.title AS title_section,
	expenses.title AS title_expense,
	expenses.description
FROM
	expenses_instances
	INNER JOIN sections ON expenses_instances.uid_section = sections.uid
	INNER JOIN expenses ON expenses_instances.uid_expense = expenses.uid
WHERE
	expenses_instances.uid_section = ?1
	AND expenses_instances.uid_expense = ?2
ORDER BY
	expenses_instances.position ASC