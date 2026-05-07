SELECT
	expense_section.uid_section,
	expense_section.uid_expense,
	sections.title AS title_section,
	expenses.title AS title_expense,
	expenses.description
FROM
	expense_section
	INNER JOIN sections ON expense_section.uid_section = sections.uid
	INNER JOIN expenses ON expense_section.uid_expense = expenses.uid
WHERE
	expense_section.uid_section = ?1
GROUP BY
	sections.uid,
	expenses.uid
ORDER BY
	expenses.position ASC