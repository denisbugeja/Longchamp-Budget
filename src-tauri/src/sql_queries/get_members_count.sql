SELECT
	COALESCE(
		(
			SELECT
				members_count
			FROM
				sections
			WHERE
				uid = ?1
		),
		0
	);