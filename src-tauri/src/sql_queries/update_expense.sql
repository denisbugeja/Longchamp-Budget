UPDATE
	expenses
SET
	title = ?1,
	description = ?2,
	rate = ?3,
	unit_price = ?4
WHERE
	uid = ?5