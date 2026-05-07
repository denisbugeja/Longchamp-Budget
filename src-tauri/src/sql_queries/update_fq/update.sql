UPDATE
	fqs
SET
	title = ?1,
	coeff = ?2,
	national_contribution = ?3,
	online_commission_rate = ?4,
	online_commission_fees = ?5
WHERE
	uid = ?6