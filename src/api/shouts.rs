use axum::extract::{Path, State};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use tracing::instrument;
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{
    models::shouts::Shout,
    util::{
        errors::{RouteError, SimpleRouteErrorOutput},
        jwt::Claims,
    },
    AppState,
};

pub fn routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new().routes(routes!(delete_shout))
}

///Delete shout by ID
#[utoipa::path(
    method(delete),
    path = "/{id}",
    params(
        ("id" = i32, Path, description = "ID of shout to delete"),
    ),
    responses(
        (status = OK, description = "Success", content_type = "application/json"),
        (status = NOT_FOUND, description = "Shout not found", body = SimpleRouteErrorOutput, content_type = "application/json"),
        (status = UNAUTHORIZED, description = "Unauthorized", body = SimpleRouteErrorOutput, content_type = "application/json"),
        (status = INTERNAL_SERVER_ERROR, description = "Miscellaneous error", body = SimpleRouteErrorOutput)
    ),
    security(
        ("token_jwt" = [])
    )
)]
#[instrument(skip(state, claims), err(Debug))]
async fn delete_shout(
    State(state): State<AppState>,
    claims: Claims,
    Path(id): Path<i32>,
) -> Result<(), RouteError> {
    use crate::schema::shouts;

    let mut conn = state.db.get().await?;

    let shout = shouts::table
        .filter(shouts::id.eq(id))
        .first::<Shout>(&mut conn)
        .await
        .optional()?
        .ok_or_else(RouteError::new_not_found)?;

    if shout.user_can_delete(claims.profile.id, &mut conn).await? {
        diesel::delete(shouts::table.filter(shouts::id.eq(id)))
            .execute(&mut conn)
            .await?;
    } else {
        return Err(RouteError::new_unauthorized());
    }

    Ok(())
}
