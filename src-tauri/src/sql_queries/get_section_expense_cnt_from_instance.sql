SELECT
	COALESCE(
		(
			SELECT
				ROUND(SUM(number), 2)
			FROM
				expenses_instances
			WHERE
				uid_section = ?1
				AND uid_expense = ?2
			GROUP BY
				expenses_instances.uid_expense
		),
		0
	);