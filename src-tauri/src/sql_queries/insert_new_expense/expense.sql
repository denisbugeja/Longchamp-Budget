INSERT INTO
	expenses (
		uid,
		title,
		description,
		rate,
		unit_price,
		position
	)
VALUES
	(
		?1,
		?2,
		?3,
		?4,
		?5,
		(
			SELECT
				COALESCE(MAX(position), -1) + 1
			FROM
				expenses
		)
	)