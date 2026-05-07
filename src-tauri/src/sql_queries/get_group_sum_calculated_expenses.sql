SELECT
	ROUND(SUM(sum_group_applyed_unit_price), 2) AS sum_group_applyed_unit_price,
	ROUND(SUM(sum_group_applyed_total_price), 2) AS sum_group_applyed_total_price
FROM
	(
		SELECT
			SUM(group_applyed_unit_price) AS sum_group_applyed_unit_price,
			SUM(group_applyed_total_price) AS sum_group_applyed_total_price
		FROM
			view_calculated_expenses_sections_instances
		WHERE
			group_rate <> 0
		UNION
		ALL
		SELECT
			SUM(total_applyed_price / group_members_count) AS sum_group_applyed_unit_price,
			SUM(total_applyed_price) AS sum_group_applyed_total_price
		FROM
			view_calculated_expenses_sections_instances
		WHERE
			uid_section = 'group'
	)