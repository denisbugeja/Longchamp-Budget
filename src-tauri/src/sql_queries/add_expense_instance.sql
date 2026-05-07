INSERT INTO
	expenses_instances (uid, uid_section, uid_expense, position)
VALUES
	(
		?1,
		?2,
		?3,
		(
			SELECT
				COALESCE(MAX(position), -1) + 1
			FROM
				expenses_instances
			WHERE
				uid_section = ?2
		)
	)