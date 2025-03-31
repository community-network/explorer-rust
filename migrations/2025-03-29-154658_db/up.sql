-- Your SQL goes here
CREATE TABLE "current_experiences"(
	"id" INT4 NOT NULL PRIMARY KEY,
	"code" INT4 NOT NULL
);

CREATE TABLE "experiences"(
	"experience_id" INT4 NOT NULL PRIMARY KEY,
	"share_code" VARCHAR(25) NOT NULL,
	"playground_name" VARCHAR(255) NOT NULL,
	"playground_description" TEXT NOT NULL,
	"playground_data" JSONB NOT NULL,
	"created_at" TIMESTAMP NOT NULL,
	"updated_at" TIMESTAMP NOT NULL,
	"playground_created_at" TIMESTAMP NOT NULL,
	"playground_updated_at" TIMESTAMP NOT NULL,
	"progression_mode" JSONB NOT NULL,
	"tags" JSONB NOT NULL
);

