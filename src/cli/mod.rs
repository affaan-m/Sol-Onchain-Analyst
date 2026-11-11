use colored::*;
use console::Term;
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

pub struct CliProgress {
    progress_bar: ProgressBar,
}

impl CliProgress {
    pub fn new(msg: &str) -> Self {
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ")
                .template("{spinner:.blue} {msg}")
                .unwrap(),
        );
        pb.set_message(msg.to_string());
        pb.enable_steady_tick(Duration::from_millis(100));
        
        Self { progress_bar: pb }
    }

    pub fn finish_with_message(&self, msg: &str) {
        self.progress_bar.finish_with_message(msg.to_string());
    }
}

pub fn print_section_header(title: &str) {
    println!("\n{}", format!("=== {} ===", title).cyan().bold());
}

pub fn print_token_info(name: &str, symbol: &str, price: f64, market_cap: f64, volume_24h: f64, price_change_24h: f64) {
    println!(
        "\n{}",
        format!("{} ({})", name, symbol).white().bold()
    );
    println!("Price: {}", format!("${:.8}", price).green());
    println!(
        "Market Cap: {}",
        format!("${:.2}M", market_cap / 1_000_000.0).yellow()
    );
    println!(
        "24h Volume: {}",
        format!("${:.2}", volume_24h).blue()
    );
    
    let price_change = if price_change_24h >= 0.0 {
        format!("+{:.2}%", price_change_24h).green()
    } else {
        format!("{:.2}%", price_change_24h).red()
    };
    println!("24h Change: {}", price_change);
}

pub fn print_analysis_summary(
    total_analyzed: i64,
    total_passed: i64,
    market_score: f64,
    social_score: f64,
    dev_score: f64,
    risk_score: f64,
) {
    print_section_header("Analysis Summary");
    
    println!(
        "Tokens: {} analyzed, {} passed filters",
        total_analyzed.to_string().yellow(),
        total_passed.to_string().green()
    );
    
    println!("\nScores:");
    print_score("Market", market_score);
    print_score("Social", social_score);
    print_score("Development", dev_score);
    print_score("Risk", risk_score);
}

fn print_score(label: &str, score: f64) {
    let (bar_char, color_fn): (&str, fn(String) -> ColoredString) = match score {
        s if s >= 0.8 => ("█", |s| s.green()),
        s if s >= 0.6 => ("█", |s| s.yellow()),
        _ => ("█", |s| s.red()),
    };
    
    let bar_length = (score * 20.0) as usize;
    let bar = format!("{}{}", bar_char.repeat(bar_length), "░".repeat(20 - bar_length));
    
    println!(
        "{}: {} {}",
        format!("{:12}", label).bold(),
        color_fn(bar),
        color_fn(format!("{:.2}", score))
    );
}

pub fn print_market_signals(
    signal_type: &str,
    confidence: f64,
    risk_score: f64,
    price_change: Option<f64>,
    volume_change: Option<f64>,
) {
    print_section_header("Market Signals");
    
    println!("Signal: {}", signal_type.magenta().bold());
    println!(
        "Confidence: {}",
        format!("{:.2}", confidence).yellow()
    );
    println!(
        "Risk Score: {}",
        format!("{:.2}", risk_score).red()
    );
    
    if let Some(price) = price_change {
        let price_str = if price >= 0.0 {
            format!("+{:.2}%", price).green()
        } else {
            format!("{:.2}%", price).red()
        };
        println!("Price Change (24h): {}", price_str);
    }
    
    if let Some(volume) = volume_change {
        let volume_str = if volume >= 0.0 {
            format!("+{:.2}%", volume).green()
        } else {
            format!("{:.2}%", volume).red()
        };
        println!("Volume Change (24h): {}", volume_str);
    }
}

pub fn clear_screen() {
    Term::stdout().clear_screen().unwrap();
} 