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
    },
    util::{
        errors::{RouteError, SimpleRouteErrorOutput},
        game_types::{Character, League},
        jwt::Claims, query::SortType,
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
}

#[derive(serde::Serialize, utoipa::ToSchema)]
struct ScoreResponse {
    #[serde(flatten)]
    score: Score,
    #[serde(skip_serializing_if = "Option::is_none")]
    player: Option<PlayerPublic>,
}

/// Get score by ID
#[utoipa::path(
    method(get),
    path = "/{id}",
    params(
        ("id" = i32, Path, description = "ID of score to get"),
        ("withPlayer" = Option<bool>, Query, description = "Include player info")
    ),
    responses(
        (status = OK, description = "Success", body = ScoreResponse, content_type = "application/json"),
        (status = NOT_FOUND, description = "Score not found", body = SimpleRouteErrorOutput, content_type = "application/json")
    )
)]
async fn get_score(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    query: Query<GetScoreParams>,
) -> Result<Json<ScoreResponse>, RouteError> {
    use crate::schema::{players, scores};

    let mut conn = state.db.get().await?;

    let score: Score = scores::table
        .find(id)
        .first::<Score>(&mut conn)
        .await
        .optional()?
        .ok_or_else(RouteError::new_not_found)?;

    let score_response = if query.with_player {
        let player: Player = players::table
            .find(score.player_id)
            .first::<Player>(&mut conn)
            .await?;

        ScoreResponse {
            score,
            player: Some(player.into()),
        }
    } else {
        ScoreResponse {
            score,
            player: None,
        }
    };

    Ok(Json(score_response))
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
    #[validate(range(min = 1))]
    #[serde_inline_default(1)]
    page: i64,
    #[validate(range(min = 1, max = 50))]
    #[serde_inline_default(10)]
    page_size: i64,
    #[serde_inline_default(Some(SortType::Desc))]
    score_sort: Option<SortType>,
    time_sort: Option<SortType>,
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
        ("page" = Option<i64>, Query, description = "Page number", minimum = 1),
        ("pageSize" = Option<i64>, Query, description = "Page size", minimum = 1, maximum = 50),
        ("scoreSort" = Option<SortType>, Query, description = "Sort by score"),
        ("timeSort" = Option<SortType>, Query, description = "Sort by submission time"),
        ("league" = Option<League>, Query, description = "League to filter by"),
        ("character" = Option<Character>, Query, description = "Character to filter by"),
        ("playerId" = Option<i32>, Query, description = "Player ID to filter by"),
    ),
    responses(
        (status = OK, description = "Success", body = ScoreResponse, content_type = "application/json"),
        (status = NOT_FOUND, description = "Song not found", body = SimpleRouteErrorOutput, content_type = "application/json")
    )
)]
async fn get_scores(
    State(state): State<AppState>,
    query: Query<GetScoresParams>,
) -> Result<Json<Vec<ScoreResponse>>, RouteError> {
    use crate::schema::{players, scores};

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
    if let Some(score_sort) = &query.score_sort {
        match score_sort {
            SortType::Asc => db_query = db_query.then_order_by(scores::score.asc()),
            SortType::Desc => db_query = db_query.then_order_by(scores::score.desc()),
        }
    }
    if let Some(time_sort) = &query.time_sort {
        match time_sort {
            SortType::Asc => db_query = db_query.then_order_by(scores::submitted_at.asc()),
            SortType::Desc => db_query = db_query.then_order_by(scores::submitted_at.desc()),
        }
    }
    db_query = db_query
        .offset((query.page - 1) * query.page_size)
        .limit(query.page_size);
    if query.with_player {
        let scores_with_player: Vec<(Score, Player)> = db_query
            .inner_join(players::table)
            .select((Score::as_select(), Player::as_select()))
            .load::<(Score, Player)>(&mut conn)
            .await?;

        let scores: Vec<ScoreResponse> = scores_with_player
            .into_iter()
            .map(|(score, player)| ScoreResponse {
                score,
                player: Some(player.into()),
            })
            .collect();

        Ok(Json(scores))
    } else {
        let scores: Vec<Score> = db_query.load::<Score>(&mut conn).await?;

        let scores: Vec<ScoreResponse> = scores
            .into_iter()
            .map(|score| ScoreResponse {
                score,
                player: None,
            })
            .collect();

        Ok(Json(scores))
    }
}
