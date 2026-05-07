INSERT INTO
	expenses_instances (
		uid,
		uid_expense,
		uid_section,
		comments,
		number,
		units,
		units_adults,
		unit_price,
		rate,
		position
	)
SELECT
	?1 AS uid,
	uid_expense,
	uid_section,
	comments,
	number,
	units,
	units_adults,
	unit_price,
	rate,
	(
		SELECT
			COALESCE(MAX(position), -1) + 1
		FROM
			expenses_instances
	) AS position
FROM
	expenses_instances
WHERE
	uid = ?2