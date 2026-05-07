SELECT
	ROUND(SUM(group_applyed_unit_price), 2) AS sum_group_applyed_unit_price,
	ROUND(SUM(group_applyed_total_price), 2) AS sum_group_applyed_total_price
FROM
	view_calculated_expenses_sections_instances
WHERE
	group_rate <> 0