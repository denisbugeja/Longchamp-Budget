SELECT
	uid_expense_instance,
	uid_section,
	uid_expense,
	title_section,
	title_expense,
	comments,
	section_color,
	expenses_units,
	expenses_units_adults,
	expenses_unit_price,
	expenses_rate,
	expenses_instances_units,
	expenses_instances_units_adults,
	expenses_instances_unit_price,
	expenses_instances_rate,
	live_units,
	live_units_adults,
	live_unit_price,
	live_rate,
	group_rate,
	applyed_price,
	total_applyed_price,
	total_inital_price,
	group_applyed_total_price,
	group_applyed_unit_price,
	group_members_count,
	expenses_description,
	expenses_instances_number
FROM
	view_calculated_expenses_sections_instances
WHERE
	uid_section = ?1
ORDER BY
	expenses_instances_position ASC