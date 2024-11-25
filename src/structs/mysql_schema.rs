// @generated automatically by Diesel CLI.

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
