UPDATE
	expenses_instances
SET
	units = ?1,
	units_adults = ?2,
	unit_price = ?3,
	rate = ?4,
	comments = ?5,
	number = ?6
WHERE
	uid = ?7