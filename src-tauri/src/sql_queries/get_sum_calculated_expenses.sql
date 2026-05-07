SELECT
	ROUND(SUM(total_applyed_price / expenses_units), 2) AS applyed_price,
	ROUND(SUM(total_applyed_price), 2) AS total_applyed_price
FROM
	view_calculated_expenses_sections_instances
WHERE
	uid_section = ?1