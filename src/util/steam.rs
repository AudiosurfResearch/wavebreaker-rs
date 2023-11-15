use steam_rs::{Steam};

pub fn init_steam(api_key: &str) {
    let steam = Steam::new(api_key);
}

pub fn verify_ticket(ticket: &str) {
    unimplemented!()
}