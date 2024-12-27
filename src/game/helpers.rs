use anyhow::{Context, Error};
use fred::prelude::{Pool as RedisPool, *};
use steam_rs::{steam_id::SteamId, Steam};

/// Validates Steam game auth tickets. Returns a `SteamId` struct representing for user who the ticket belongs to.
/// Checks if the ticket is cached in Redis, if not, it will authenticate with Steam and cache the ticket.
/// 
/// # Errors
/// This function will return an error if it fails to authenticate with Steam or something goes wrong with Redis.
pub async fn ticket_auth(ticket: &str, steam: &Steam, redis: &RedisPool) -> Result<SteamId, Error> {
    let steam_id: Option<String> = redis.get(format!("steamticket:{}", ticket)).await?;
    match steam_id {
        Some(steam_id) => {
            Ok(SteamId::from(steam_id))
        }
        None => {
            let steam_result = steam
                .authenticate_user_ticket(12900, ticket)
                .await
                .context("Failed to authenticate with Steam")?;
            redis.set::<(), _, _>(format!("steamticket:{}", ticket), &steam_result.steam_id, Some(Expiration::EX(60 * 60 * 8)), None, false).await?;

            Ok(SteamId::from(steam_result.steam_id))
        }
    }
}
