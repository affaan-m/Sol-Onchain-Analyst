

mod get_token_data_by_address;
pub use get_token_data_by_address::get_token_data_by_address;

mod trade;
pub use trade::trade;

mod fetch_price;
pub use fetch_price::fetch_price;

mod stake_with_jup;
pub use stake_with_jup::stake_with_jup;

/// Jupiter API URL
pub const JUP_API: &str = "https://quote-api.jup.ag/v6";
pub const JUP_REFERRAL_ADDRESS: &str = "REFER4ZgmyYx9c6He5XfaTMiGfdLwRnkV4RPp9t9iF3";
pub const JUP_PRICE_V2: &str = "https://api.jup.ag/price/v2?ids=";
