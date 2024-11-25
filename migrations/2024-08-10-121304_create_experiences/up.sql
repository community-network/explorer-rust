-- Your SQL goes here
CREATE TABLE `experiences` (
    `experience_id` INT UNSIGNED NOT NULL PRIMARY KEY,
    `share_code` VARCHAR(25) NOT NULL,
    `playground_name` VARCHAR(255) NOT NULL,
    `playground_description` TEXT NOT NULL,

    `playground_data` JSON NOT NULL,

    `created_at` TIMESTAMP NOT NULL DEFAULT NOW(),
    `updated_at` TIMESTAMP NOT NULL DEFAULT NOW() ON UPDATE now()
)