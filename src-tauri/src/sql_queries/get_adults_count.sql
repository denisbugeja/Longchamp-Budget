SELECT
COALESCE(
	(
		SELECT
			COALESCE(adults_count, 0)
		FROM
			sections
		WHERE
			uid = ?1
),
0
);