BEGIN;

CREATE TABLE IF NOT EXISTS "sections" (
	"uid" TEXT NOT NULL UNIQUE,
	"title" TEXT NOT NULL,
	"color" TEXT,
	"members_count" NUMERIC NOT NULL DEFAULT 0,
	"adults_count" NUMERIC NOT NULL DEFAULT 0,
	"position" INTEGER NOT NULL DEFAULT 0,
	PRIMARY KEY("uid"),
	UNIQUE("title")
);

CREATE TABLE IF NOT EXISTS "expenses" (
	"uid" TEXT NOT NULL UNIQUE,
	"title" TEXT NOT NULL,
	"description" TEXT,
	"rate" NUMERIC NOT NULL DEFAULT 100,
	"unit_price" NUMERIC NOT NULL DEFAULT 0,
	"position" INTEGER NOT NULL DEFAULT 0,
	PRIMARY KEY("uid")
);

CREATE TABLE IF NOT EXISTS "expense_section" (
	"uid_expense" TEXT NOT NULL,
	"uid_section" TEXT NOT NULL,
	FOREIGN KEY("uid_expense") REFERENCES "expenses"("uid"),
	FOREIGN KEY("uid_section") REFERENCES "sections"("uid"),
	UNIQUE("uid_expense", "uid_section")
);

CREATE TABLE IF NOT EXISTS "expenses_instances" (
	"uid" TEXT NOT NULL UNIQUE,
	"uid_expense" TEXT NOT NULL,
	"uid_section" TEXT NOT NULL,
	"comments" TEXT,
	"number" NUMERIC NOT NULL DEFAULT 1,
	"units" NUMERIC,
	"units_adults" NUMERIC,
	"unit_price" NUMERIC,
	"rate" NUMERIC,
	"position" INTEGER NOT NULL DEFAULT 0,
	FOREIGN KEY("uid_expense") REFERENCES "expenses"("uid"),
	FOREIGN KEY("uid_section") REFERENCES "sections"("uid"),
	PRIMARY KEY("uid")
);

CREATE TABLE IF NOT EXISTS "fqs" (
	"uid" TEXT NOT NULL UNIQUE,
	"title" TEXT NOT NULL,
	"national_contribution" NUMERIC NOT NULL DEFAULT 0,
	"coeff" NUMERIC NOT NULL DEFAULT 0,
	"online_commission_rate" NUMERIC NOT NULL DEFAULT 0,
	"online_commission_fees" NUMERIC NOT NULL DEFAULT 0,
	"position" INTEGER NOT NULL DEFAULT 0,
	PRIMARY KEY("uid"),
	UNIQUE("title")
);

CREATE TABLE IF NOT EXISTS "sections_fqs" (
	"uid_section" TEXT NOT NULL,
	"uid_fq" TEXT NOT NULL,
	"members_count" NUMERIC NOT NULL DEFAULT 0,
	FOREIGN KEY("uid_section") REFERENCES "sections"("uid"),
	FOREIGN KEY("uid_fq") REFERENCES "fqs"("uid"),
	UNIQUE("uid_section", "uid_fq")
);

CREATE INDEX IF NOT EXISTS "IX_EXPENSE_SECTION_UID_EXPENSE" ON "expense_section" ("uid_expense");

CREATE INDEX IF NOT EXISTS "IX_EXPENSE_SECTION_UID_SECTION" ON "expense_section" ("uid_section");

CREATE INDEX IF NOT EXISTS "IX_EXPENSES_INSTANCES_UID_SECTION" ON "expenses_instances" ("uid_section");

CREATE INDEX IF NOT EXISTS "IX_EXPENSES_INSTANCES_UID_EXPENSE" ON "expenses_instances" ("uid_expense");

CREATE INDEX IF NOT EXISTS "IX_SECTIONS_FQS_UID_SECTION" ON "sections_fqs" ("uid_section");

CREATE INDEX IF NOT EXISTS "IX_SECTIONS_FQS_UID_FQ" ON "sections_fqs" ("uid_fq");

INSERT INTO
	sections (uid, title, color, position)
SELECT
	'group',
	'Groupe',
	'#403f6f',
	0
WHERE
	NOT EXISTS(
		SELECT
			uid,
			title,
			color,
			position
		FROM
			sections
		WHERE
			uid = 'group'
	);

DROP VIEW IF EXISTS view_expenses_sections_instances;

CREATE VIEW view_expenses_sections_instances AS
SELECT
	expenses_instances.uid AS uid_expense_instance,
	expenses_instances.uid_section,
	expenses_instances.uid_expense,
	sections.title AS title_section,
	sections.position AS sections_position,
	expenses.title AS title_expense,
	expenses.position AS expenses_position,
	expenses_instances.comments AS comments,
	expenses_instances.position AS expenses_instances_position,
	expenses_instances.number AS expenses_instances_number,
	sections.color AS section_color,
	sections.members_count AS expenses_units,
	sections.adults_count AS expenses_units_adults,
	expenses.unit_price AS expenses_unit_price,
	expenses.rate AS expenses_rate,
	expenses.description AS expenses_description,
	expenses_instances.units AS expenses_instances_units,
	expenses_instances.units_adults AS expenses_instances_units_adults,
	expenses_instances.unit_price AS expenses_instances_unit_price,
	expenses_instances.rate AS expenses_instances_rate,
	CASE
		WHEN expenses_instances.units IS NOT NULL
		AND TRIM(expenses_instances.units, " ") != "" THEN expenses_instances.units
		ELSE sections.members_count
	END AS live_units,
	CASE
		WHEN expenses_instances.units_adults IS NOT NULL
		AND TRIM(expenses_instances.units_adults, " ") != "" THEN expenses_instances.units_adults
		ELSE sections.adults_count
	END AS live_units_adults,
	CASE
		WHEN expenses_instances.unit_price IS NOT NULL
		AND TRIM(expenses_instances.unit_price, " ") != "" THEN CAST(expenses_instances.unit_price AS REAL)
		ELSE CAST(expenses.unit_price AS REAL)
	END AS live_unit_price,
	CASE
		WHEN expenses_instances.rate IS NOT NULL
		AND TRIM(expenses_instances.rate, " ") != "" THEN CAST(expenses_instances.rate AS REAL)
		ELSE CAST(expenses.rate AS REAL)
	END AS live_rate,
	CASE
		WHEN group_sections.members_count > 0 THEN group_sections.members_count
		ELSE 1
	END AS group_members_count,
	CASE
		WHEN group_sections.adults_count > 0 THEN group_sections.adults_count
		ELSE 1
	END AS group_adults_count
FROM
	sections AS group_sections,
	expenses_instances
	INNER JOIN sections ON expenses_instances.uid_section = sections.uid
	INNER JOIN expenses ON expenses_instances.uid_expense = expenses.uid
WHERE
	group_sections.uid = 'group';

DROP VIEW IF EXISTS view_calculated_expenses_sections_instances;

CREATE VIEW view_calculated_expenses_sections_instances AS
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
	live_unit_price,
	live_rate,
	(100 - view_expenses_sections_instances.live_rate) AS group_rate,
	ROUND(
		view_expenses_sections_instances.live_unit_price * (view_expenses_sections_instances.live_rate / 100),
		2
	) AS applyed_price,
	ROUND(
		(
			view_expenses_sections_instances.expenses_instances_number * (
				view_expenses_sections_instances.live_units + view_expenses_sections_instances.live_units_adults
			)
		) * view_expenses_sections_instances.live_unit_price * (view_expenses_sections_instances.live_rate / 100),
		2
	) AS total_applyed_price,
	ROUND(
		(
			view_expenses_sections_instances.expenses_instances_number * (
				view_expenses_sections_instances.live_units + view_expenses_sections_instances.live_units_adults
			)
		) * view_expenses_sections_instances.live_unit_price,
		2
	) AS total_inital_price,
	ROUND(
		(
			view_expenses_sections_instances.expenses_instances_number * (
				view_expenses_sections_instances.live_units + view_expenses_sections_instances.live_units_adults
			)
		) * view_expenses_sections_instances.live_unit_price - (
			view_expenses_sections_instances.expenses_instances_number * (
				view_expenses_sections_instances.live_units + view_expenses_sections_instances.live_units_adults
			)
		) * view_expenses_sections_instances.live_unit_price * (view_expenses_sections_instances.live_rate / 100),
		2
	) AS group_applyed_total_price,
	ROUND(
		(
			(
				(
					view_expenses_sections_instances.expenses_instances_number * (
						view_expenses_sections_instances.live_units + view_expenses_sections_instances.live_units_adults
					)
				) * view_expenses_sections_instances.live_unit_price - (
					view_expenses_sections_instances.expenses_instances_number * (
						view_expenses_sections_instances.live_units + view_expenses_sections_instances.live_units_adults
					)
				) * view_expenses_sections_instances.live_unit_price * (view_expenses_sections_instances.live_rate / 100)
			) / group_members_count
		),
		2
	) AS group_applyed_unit_price,
	group_members_count,
	group_adults_count,
	live_units_adults,
	expenses_instances_position,
	expenses_position,
	sections_position,
	expenses_description,
	expenses_instances_number
FROM
	view_expenses_sections_instances;

DROP TRIGGER IF EXISTS update_group_members_count_after_update;

CREATE TRIGGER update_group_members_count_after_update
AFTER
UPDATE
	ON sections FOR EACH ROW BEGIN
UPDATE
	sections
SET
	members_count = COALESCE(
		(
			SELECT
				SUM(COALESCE(members_count, 0))
			FROM
				sections
			WHERE
				uid != 'group'
		),
		0
	),
	adults_count = COALESCE(
		(
			SELECT
				SUM(COALESCE(adults_count, 0))
			FROM
				sections
			WHERE
				uid != 'group'
		),
		0
	)
WHERE
	uid = 'group';

END;

DROP TRIGGER IF EXISTS update_group_members_count_after_insert;

CREATE TRIGGER update_group_members_count_after_insert
AFTER
INSERT
	ON sections FOR EACH ROW BEGIN
UPDATE
	sections
SET
	members_count = COALESCE(
		(
			SELECT
				SUM(COALESCE(members_count, 0))
			FROM
				sections
			WHERE
				uid != 'group'
		),
		0
	),
	adults_count = COALESCE(
		(
			SELECT
				SUM(COALESCE(adults_count, 0))
			FROM
				sections
			WHERE
				uid != 'group'
		),
		0
	)
WHERE
	uid = 'group';

END;

DROP TRIGGER IF EXISTS update_group_members_count_after_delete;

CREATE TRIGGER update_group_members_count_after_delete
AFTER
	DELETE ON sections FOR EACH ROW BEGIN
UPDATE
	sections
SET
	members_count = COALESCE(
		(
			SELECT
				SUM(COALESCE(members_count, 0))
			FROM
				sections
			WHERE
				uid != 'group'
		),
		0
	),
	adults_count = COALESCE(
		(
			SELECT
				SUM(COALESCE(adults_count, 0))
			FROM
				sections
			WHERE
				uid != 'group'
		),
		0
	)
WHERE
	uid = 'group';

END;

DROP TRIGGER IF EXISTS insert_sections_fqs_after_insert_sections;

CREATE TRIGGER insert_sections_fqs_after_insert_sections
AFTER
INSERT
	ON sections FOR EACH ROW BEGIN
INSERT INTO
	sections_fqs (uid_section, uid_fq, members_count)
SELECT
	sections.uid AS uid_section,
	fqs.uid AS uid_fq,
	0
FROM
	sections,
	fqs
WHERE
	sections.uid = NEW.uid;

END;

DROP TRIGGER IF EXISTS insert_sections_fqs_after_insert_fqs;

CREATE TRIGGER insert_sections_fqs_after_insert_fqs
AFTER
INSERT
	ON fqs FOR EACH ROW BEGIN
INSERT INTO
	sections_fqs (uid_section, uid_fq, members_count)
SELECT
	sections.uid AS uid_section,
	fqs.uid AS uid_fq,
	0
FROM
	sections,
	fqs
WHERE
	fqs.uid = NEW.uid;

END;

DROP TRIGGER IF EXISTS delete_sections_fqs_after_delete_sections;

CREATE TRIGGER delete_sections_fqs_after_delete_sections BEFORE DELETE ON sections FOR EACH ROW BEGIN
DELETE FROM
	sections_fqs
WHERE
	uid_section = OLD.uid;

UPDATE
	sections_fqs
SET
	members_count = (
		SELECT
			SUM(COALESCE(members_count, 0))
		FROM
			sections_fqs s2
		WHERE
			s2.uid_section != 'group'
			AND s2.uid_fq = sections_fqs.uid_fq
	)
WHERE
	uid_section = 'group';

END;

DROP TRIGGER IF EXISTS update_sections_fqs_after_update_sections_fqs;

CREATE TRIGGER update_sections_fqs_after_update_sections_fqs
AFTER
UPDATE
	ON sections_fqs FOR EACH ROW BEGIN
UPDATE
	sections_fqs
SET
	members_count = (
		SELECT
			SUM(COALESCE(members_count, 0))
		FROM
			sections_fqs s2
		WHERE
			s2.uid_section != 'group'
			AND s2.uid_fq = sections_fqs.uid_fq
	)
WHERE
	uid_section = 'group';

END;

DROP TRIGGER IF EXISTS delete_sections_fqs_after_delete_fqs;

CREATE TRIGGER delete_sections_fqs_after_delete_fqs BEFORE DELETE ON fqs FOR EACH ROW BEGIN
DELETE FROM
	sections_fqs
WHERE
	uid_fq = OLD.uid;

UPDATE
	sections_fqs
SET
	members_count = (
		SELECT
			SUM(COALESCE(members_count, 0))
		FROM
			sections_fqs s2
		WHERE
			s2.uid_section != 'group'
			AND s2.uid_fq = sections_fqs.uid_fq
	)
WHERE
	uid_section = 'group';

END;

DROP VIEW IF EXISTS view_declared_sections_fq_members;

CREATE VIEW view_declared_sections_fq_members AS
SELECT
	uid_section,
	COALESCE(SUM(members_count), 0) AS total_members_fq_declared
FROM
	sections_fqs
GROUP BY
	uid_section;

DROP VIEW IF EXISTS view_declared_fqs_sections_total_price;

CREATE VIEW view_declared_fqs_sections_total_price AS
SELECT
	view_declared_sections_fq_members.uid_section,
	ROUND(
		SUM(
			total_applyed_price / expenses_units * view_declared_sections_fq_members.total_members_fq_declared
		),
		2
	) AS total_declared
FROM
	view_calculated_expenses_sections_instances
	INNER JOIN view_declared_sections_fq_members ON view_calculated_expenses_sections_instances.uid_section = view_declared_sections_fq_members.uid_section
GROUP BY
	view_declared_sections_fq_members.uid_section;

DROP VIEW IF EXISTS view_declared_fqs_sections_total_members;

CREATE VIEW view_declared_fqs_sections_total_members AS
SELECT
	sections_fqs.uid_section,
	ROUND(
		COALESCE(SUM(fqs.coeff * sections_fqs.members_count), 0),
		2
	) as fqs_total_members
FROM
	sections_fqs
	INNER JOIN fqs ON sections_fqs.uid_fq = fqs.uid
GROUP BY
	sections_fqs.uid_section;

DROP VIEW IF EXISTS view_declared_fqs_group_unit_price;

CREATE VIEW view_declared_fqs_group_unit_price AS
SELECT
	'group' AS uid_section,
	ROUND(
		SUM(COALESCE(sum_group_applyed_unit_price, 0)),
		2
	) AS declared_unit_price
FROM
	(
		SELECT
			SUM(group_applyed_unit_price) AS sum_group_applyed_unit_price
		FROM
			view_calculated_expenses_sections_instances
		WHERE
			group_rate <> 0
		UNION
		ALL
		SELECT
			SUM(total_applyed_price / group_members_count) AS sum_group_applyed_unit_price
		FROM
			view_calculated_expenses_sections_instances
		WHERE
			uid_section = 'group'
	);

DROP VIEW IF EXISTS view_declared_fqs_sections_unit_price;

CREATE VIEW view_declared_fqs_sections_unit_price AS
SELECT
	view_declared_fqs_sections_total_price.uid_section,
	ROUND(
		view_declared_fqs_sections_total_price.total_declared / view_declared_fqs_sections_total_members.fqs_total_members,
		2
	) AS declared_unit_price
FROM
	view_declared_fqs_sections_total_price
	INNER JOIN view_declared_fqs_sections_total_members ON view_declared_fqs_sections_total_price.uid_section = view_declared_fqs_sections_total_members.uid_section
WHERE
	view_declared_fqs_sections_total_price.uid_section <> 'group'
UNION
ALL
SELECT
	view_declared_fqs_group_unit_price.uid_section,
	ROUND(
		view_declared_fqs_group_unit_price.declared_unit_price * view_declared_sections_fq_members.total_members_fq_declared / view_declared_fqs_sections_total_members.fqs_total_members,
		2
	) AS declared_unit_price
FROM
	view_declared_fqs_group_unit_price
	INNER JOIN view_declared_fqs_sections_total_members ON view_declared_fqs_group_unit_price.uid_section = view_declared_fqs_sections_total_members.uid_section
	INNER JOIN view_declared_sections_fq_members ON view_declared_fqs_group_unit_price.uid_section = view_declared_sections_fq_members.uid_section;

DROP VIEW IF EXISTS view_declared_calculated_fqs_sections_unit_price;

CREATE VIEW view_declared_calculated_fqs_sections_unit_price AS
SELECT
	sections_fqs.uid_fq,
	sections_fqs.uid_section,
	view_declared_fqs_sections_unit_price.declared_unit_price,
	ROUND(fqs.coeff, 2) as coeff,
	ROUND(
		view_declared_fqs_sections_unit_price.declared_unit_price * fqs.coeff,
		2
	) AS calculated_unit_price_with_coeff
FROM
	view_declared_fqs_sections_unit_price
	INNER JOIN sections_fqs ON view_declared_fqs_sections_unit_price.uid_section = sections_fqs.uid_section
	INNER JOIN fqs ON sections_fqs.uid_fq = fqs.uid;

DROP VIEW IF EXISTS view_calculated_fqs_total;

CREATE VIEW view_calculated_fqs_total AS
SELECT
	sections.title as title_section,
	fqs.title as title_fq,
	s.uid_fq,
	s.uid_section,
	s.declared_unit_price,
	g.declared_unit_price as declared_group_unit_price,
	s.coeff,
	s.calculated_unit_price_with_coeff,
	g.calculated_unit_price_with_coeff AS group_calculated_unit_price,
	ROUND(
		s.calculated_unit_price_with_coeff + g.calculated_unit_price_with_coeff,
		2
	) AS total_group_member_price,
	fqs.national_contribution,
	ROUND(
		s.calculated_unit_price_with_coeff + g.calculated_unit_price_with_coeff + fqs.national_contribution,
		2
	) AS total_member_price,
	ROUND(
		(
			s.calculated_unit_price_with_coeff + g.calculated_unit_price_with_coeff + fqs.national_contribution
		) * fqs.online_commission_rate + fqs.online_commission_fees,
		2
	) AS national_commission,
	ROUND(
		s.calculated_unit_price_with_coeff + g.calculated_unit_price_with_coeff + fqs.national_contribution,
		2
	) + ROUND(
		(
			s.calculated_unit_price_with_coeff + g.calculated_unit_price_with_coeff + fqs.national_contribution
		) * fqs.online_commission_rate + fqs.online_commission_fees,
		2
	) AS total,
	sections_fqs.members_count AS members_declared_count,
	sections.color as color
FROM
	view_declared_calculated_fqs_sections_unit_price AS s
	INNER JOIN view_declared_calculated_fqs_sections_unit_price AS g ON s.uid_fq = g.uid_fq
	INNER JOIN fqs ON s.uid_fq = fqs.uid
	INNER JOIN sections_fqs ON s.uid_fq = sections_fqs.uid_fq
	AND s.uid_section = sections_fqs.uid_section
	INNER JOIN sections ON s.uid_section = sections.uid
	AND g.uid_section = 'group'
ORDER BY
	sections.position,
	fqs.position ASC;

COMMIT;