// @generated automatically by Diesel CLI.

diesel::table! {
    current_experiences (id) {
        id -> Int4,
        code -> Int4,
    }
}

diesel::table! {
    experiences (experience_id) {
        experience_id -> Int4,
        #[max_length = 25]
        share_code -> Varchar,
        #[max_length = 255]
        playground_name -> Varchar,
        playground_description -> Text,
        playground_data -> Jsonb,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        playground_created_at -> Timestamp,
        playground_updated_at -> Timestamp,
        progression_mode -> Jsonb,
        tags -> Jsonb,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    current_experiences,
    experiences,
);
