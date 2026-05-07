INSERT INTO
	fqs (
		uid,
		title,
		coeff,
		national_contribution,
		online_commission_rate,
		online_commission_fees,
		position
	)
VALUES
	(
		?1,
		?2,
		?3,
		?4,
		?5,
		?6,
		(
			SELECT
				COALESCE(MAX(position), -1) + 1
			FROM
				fqs
		)
	);