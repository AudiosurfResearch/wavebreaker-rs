use axum::{
    extract::{Path, Query, State},
    Json,
};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use serde::Deserialize;
use serde_inline_default::serde_inline_default;
use utoipa_axum::{router::OpenApiRouter, routes};
use validator::Validate;

use crate::{
    models::{
        players::{AccountType, Player, PlayerPublic},
        scores::Score,
        songs::Song,
    },
    util::{
        errors::{RouteError, SimpleRouteErrorOutput},
        game_types::{Character, League},
        jwt::Claims,
        query::SortType,
    },
    AppState,
};

pub fn routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(get_score, delete_score))
        .routes(routes!(get_scores))
}

#[serde_inline_default]
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct GetScoreParams {
    #[serde_inline_default(true)]
    with_player: bool,
    #[serde_inline_default(true)]
    with_song: bool,
}

#[derive(serde::Serialize, utoipa::ToSchema)]
struct ScoreSearchResponse {
    results: Vec<ScoreSearchResult>,
    total: i64,
}

#[derive(serde::Serialize, utoipa::ToSchema)]
struct ScoreSearchResult {
    #[serde(flatten)]
    score: Score,
    #[serde(skip_serializing_if = "Option::is_none")]
    player: Option<PlayerPublic>,
    #[serde(skip_serializing_if = "Option::is_none")]
    song: Option<Song>,
}

/// Get score by ID
#[utoipa::path(
    method(get),
    path = "/{id}",
    params(
        ("id" = i32, Path, description = "ID of score to get"),
        ("withPlayer" = Option<bool>, Query, description = "Include player info"),
        ("withSong" = Option<bool>, Query, description = "Include song info"),
    ),
    responses(
        (status = OK, description = "Success", body = ScoreSearchResult, content_type = "application/json"),
        (status = NOT_FOUND, description = "Score not found", body = SimpleRouteErrorOutput, content_type = "application/json")
    )
)]
async fn get_score(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    query: Query<GetScoreParams>,
) -> Result<Json<ScoreSearchResult>, RouteError> {
    use crate::schema::{players, scores, songs};

    let mut conn = state.db.get().await?;

    let score: Score = scores::table
        .find(id)
        .first::<Score>(&mut conn)
        .await
        .optional()?
        .ok_or_else(RouteError::new_not_found)?;

    let player = if query.with_player {
        let player: Player = players::table
            .find(score.player_id)
            .first::<Player>(&mut conn)
            .await?;

        Some(player.into())
    } else {
        None
    };

    let song = if query.with_song {
        let song = songs::table
            .find(score.song_id)
            .first::<Song>(&mut conn)
            .await?;

        Some(song)
    } else {
        None
    };

    Ok(Json(ScoreSearchResult {
        score,
        player,
        song,
    }))
}

///Delete score by ID
#[utoipa::path(
    method(delete),
    path = "/{id}",
    params(
        ("id" = i32, Path, description = "ID of score to delete"),
    ),
    responses(
        (status = OK, description = "Success", content_type = "application/json"),
        (status = NOT_FOUND, description = "Score not found", body = SimpleRouteErrorOutput, content_type = "application/json"),
        (status = UNAUTHORIZED, description = "Unauthorized", body = SimpleRouteErrorOutput, content_type = "application/json"),
    ),
    security(
        ("token_jwt" = [])
    )
)]
async fn delete_score(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    claims: Claims,
) -> Result<(), RouteError> {
    use crate::schema::scores;

    if claims.profile.account_type == AccountType::Moderator
        || claims.profile.account_type == AccountType::Team
    {
        let mut conn = state.db.get().await?;

        let score: Score = scores::table
            .find(id)
            .first(&mut conn)
            .await
            .optional()?
            .ok_or_else(RouteError::new_not_found)?;
        score.delete(&mut conn, &state.redis).await?;

        Ok(())
    } else {
        Err(RouteError::new_unauthorized())
    }
}

#[serde_inline_default]
#[derive(Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
struct GetScoresParams {
    #[serde_inline_default(false)]
    with_player: bool,
    #[serde_inline_default(true)]
    with_song: bool,
    #[validate(range(min = 1))]
    #[serde_inline_default(1)]
    page: i64,
    #[validate(range(min = 1, max = 50))]
    #[serde_inline_default(10)]
    page_size: i64,
    time_sort: Option<SortType>,
    score_sort: Option<SortType>,
    league: Option<League>,
    character: Option<Character>,
    player_id: Option<i32>,
}

/// Search for scores
#[utoipa::path(
    method(get),
    path = "/",
    params(
        ("withPlayer" = Option<bool>, Query, description = "Include player info"),
        ("withSong" = Option<bool>, Query, description = "Include song info"),
        ("page" = Option<i64>, Query, description = "Page number", minimum = 1),
        ("pageSize" = Option<i64>, Query, description = "Page size", minimum = 1, maximum = 50),
        ("timeSort" = Option<SortType>, Query, description = "Sort by submission time"),
        ("scoreSort" = Option<SortType>, Query, description = "Sort by score"),
        ("league" = Option<League>, Query, description = "League to filter by"),
        ("character" = Option<Character>, Query, description = "Character to filter by"),
        ("playerId" = Option<i32>, Query, description = "Player ID to filter by"),
    ),
    responses(
        (status = OK, description = "Success", body = ScoreSearchResponse, content_type = "application/json"),
        (status = NOT_FOUND, description = "Song not found", body = SimpleRouteErrorOutput, content_type = "application/json")
    )
)]
async fn get_scores(
    State(state): State<AppState>,
    query: Query<GetScoresParams>,
) -> Result<Json<ScoreSearchResponse>, RouteError> {
    use crate::schema::{players, scores, songs};

    let mut conn = state.db.get().await?;

    let mut db_query = scores::table.into_boxed();
    if let Some(league) = query.league {
        db_query = db_query.filter(scores::league.eq(league));
    }
    if let Some(character) = query.character {
        db_query = db_query.filter(scores::vehicle.eq(character));
    }
    if let Some(player_id) = query.player_id {
        db_query = db_query.filter(scores::player_id.eq(player_id));
    }

    if let Some(time_sort) = &query.time_sort {
        match time_sort {
            SortType::Asc => db_query = db_query.then_order_by(scores::submitted_at.asc()),
            SortType::Desc => db_query = db_query.then_order_by(scores::submitted_at.desc()),
        }
    }
    if let Some(score_sort) = &query.score_sort {
        match score_sort {
            SortType::Asc => db_query = db_query.then_order_by(scores::score.asc()),
            SortType::Desc => db_query = db_query.then_order_by(scores::score.desc()),
        }
    }
    db_query = db_query
        .offset((query.page - 1) * query.page_size)
        .limit(query.page_size);

    let mut total_count_query = scores::table.into_boxed();
    if let Some(league) = query.league {
        total_count_query = total_count_query.filter(scores::league.eq(league));
    }
    if let Some(character) = query.character {
        total_count_query = total_count_query.filter(scores::vehicle.eq(character));
    }
    if let Some(player_id) = query.player_id {
        total_count_query = total_count_query.filter(scores::player_id.eq(player_id));
    }
    let total: i64 = total_count_query.count().get_result(&mut conn).await?;

    //FIXME This is messed up. What. Is there a better way to do this???
    //I don't get to dynamically join stuff or change selects because it changes the return type
    match (query.with_player, query.with_song) {
        (true, true) => {
            let items: Vec<(Score, Player, Song)> = db_query
                .inner_join(players::table)
                .inner_join(songs::table)
                .select((Score::as_select(), Player::as_select(), Song::as_select()))
                .load(&mut conn)
                .await?;

            let results = items
                .into_iter()
                .map(|(score, player, song)| ScoreSearchResult {
                    score,
                    player: Some(player.into()),
                    song: Some(song),
                })
                .collect();
            Ok(Json(ScoreSearchResponse { results, total }))
        }
        (true, false) => {
            let items: Vec<(Score, Player)> = db_query
                .inner_join(players::table)
                .select((Score::as_select(), Player::as_select()))
                .load(&mut conn)
                .await?;

            let results = items
                .into_iter()
                .map(|(score, player)| ScoreSearchResult {
                    score,
                    player: Some(player.into()),
                    song: None,
                })
                .collect();
            Ok(Json(ScoreSearchResponse { results, total }))
        }
        (false, true) => {
            let items: Vec<(Score, Song)> = db_query
                .inner_join(songs::table)
                .select((Score::as_select(), Song::as_select()))
                .load(&mut conn)
                .await?;

            let results = items
                .into_iter()
                .map(|(score, song)| ScoreSearchResult {
                    score,
                    player: None,
                    song: Some(song),
                })
                .collect();
            Ok(Json(ScoreSearchResponse { results, total }))
        }
        (false, false) => {
            let scores_only: Vec<Score> = db_query.load(&mut conn).await?;
            let results = scores_only
                .into_iter()
                .map(|score| ScoreSearchResult {
                    score,
                    player: None,
                    song: None,
                })
                .collect();
            Ok(Json(ScoreSearchResponse { results, total }))
        }
    }
}
