SELECT
	COALESCE(
		SUM(national_contribution * members_declared_count),
		0
	) AS total_national_contribution,
	COALESCE(
		SUM(national_commission * members_declared_count),
		0
	) AS total_national_commission
FROM
	view_calculated_fqs_total
WHERE
	uid_section <> 'group'