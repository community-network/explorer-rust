// @generated automatically by Diesel CLI.

diesel::table! {
    current_experiences (id) {
        id -> Integer,
        code -> Unsigned<Integer>,
    }
}

diesel::table! {
    experiences (experience_id) {
        experience_id -> Unsigned<Integer>,
        #[max_length = 25]
        share_code -> Varchar,
        #[max_length = 255]
        playground_name -> Varchar,
        playground_description -> Text,
        playground_data -> Json,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    current_experiences,
    experiences,
);
