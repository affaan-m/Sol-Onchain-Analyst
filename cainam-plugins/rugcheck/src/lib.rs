

mod token_report_summary;
pub use token_report_summary::{fetch_summary_report, TokenCheck};

mod token_report_detailed;
pub use token_report_detailed::fetch_detailed_report;

pub const RUGCHECK_URL: &str = "https://api.rugcheck.xyz/v1";
