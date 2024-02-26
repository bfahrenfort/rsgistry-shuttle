ALTER TABLE programs
DROP CONSTRAINT programs_pkey;

ALTER TABLE programs
ADD CONSTRAINT programs_pkey PRIMARY KEY (id, doctype);
