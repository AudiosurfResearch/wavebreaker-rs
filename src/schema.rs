// @generated automatically by Diesel CLI.

diesel::table! {
    players (id) {
        id -> Int4,
        #[max_length = 32]
        username -> Varchar,
        steam_id -> Text,
        steam_account_num -> Int4,
        location_id -> Int4,
        account_type -> Int2,
        joined_at -> Timestamp,
        avatar_url -> Text,
    }
}

diesel::table! {
    rivalries (player_id, rival_id) {
        player_id -> Int4,
        rival_id -> Int4,
        established_at -> Timestamp,
    }
}

diesel::allow_tables_to_appear_in_same_query!(players, rivalries,);
