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
    rivalries (challenger_id, rival_id) {
        challenger_id -> Int4,
        rival_id -> Int4,
        established_at -> Timestamp,
    }
}

diesel::table! {
    scores (id) {
        id -> Int4,
        song_id -> Int4,
        player_id -> Int4,
        league -> Int2,
        submitted_at -> Timestamp,
        play_count -> Int4,
        score -> Int4,
        track_shape -> Array<Int4>,
        xstats -> Array<Int4>,
        density -> Int4,
        vehicle -> Int2,
        feats -> Array<Text>,
        song_length -> Int4,
        gold_threshold -> Int4,
        iss -> Int4,
        isj -> Int4,
    }
}

diesel::table! {
    shouts (id) {
        id -> Int4,
        author_id -> Int4,
        song_id -> Int4,
        posted_at -> Timestamp,
        #[max_length = 240]
        content -> Varchar,
    }
}

diesel::table! {
    songs (id) {
        id -> Int4,
        title -> Text,
        artist -> Text,
        created_at -> Timestamp,
    }
}

diesel::joinable!(scores -> players (player_id));
diesel::joinable!(scores -> songs (song_id));
diesel::joinable!(shouts -> players (author_id));
diesel::joinable!(shouts -> songs (song_id));

diesel::allow_tables_to_appear_in_same_query!(players, rivalries, scores, shouts, songs,);
