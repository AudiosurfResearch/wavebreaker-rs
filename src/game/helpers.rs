use steam_rs::{steam_id::SteamId, Steam};

use crate::error::{IntoHttp, RouteError};

/// Validates Steam game auth tickets. Returns a `SteamId` struct representing for user who the ticket belongs to.
///
/// # Errors
///
/// This function will return an error if it fails to authenticate with Steam.
pub async fn ticket_auth(ticket: &str, steam: &Steam) -> Result<SteamId, RouteError> {
    let steam_result = steam
        .authenticate_user_ticket(12900, ticket)
        .await
        .http_internal_error("Failed to authenticate with Steam.")?;
    Ok(SteamId::from(steam_result.steam_id))
}
