SELECT
	uid,
	title,
	coeff,
	national_contribution,
	online_commission_rate,
	online_commission_fees
FROM
	fqs
WHERE
	title = ?1