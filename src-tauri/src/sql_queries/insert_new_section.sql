INSERT INTO
	sections (
		uid,
		title,
		color,
		members_count,
		adults_count,
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
				sections
		)
	)