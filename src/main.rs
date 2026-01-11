use dotenvy::dotenv;
use reqwest::{Client, Proxy};
use std::env;
use std::process::Command;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load .env file
    dotenv().ok();
    let _ = send_telegram_alert( "Daemon Started").await;
    
    let check_interval = Duration::from_secs(300);
    println!("🚀 Proxy Watcher with .env config started...");

    loop {
        if let Err(e) = perform_check().await {
            eprintln!("⚠️ Error: {}", e);
        }
        sleep(check_interval).await;
    }
}

async fn perform_check() -> anyhow::Result<()> {
    // Read variables from environment
    let socks_url = env::var("SOCKS5_URL")?;
    
    let proxy = Proxy::all(socks_url)?;
    let client = Client::builder()
        .proxy(proxy)
        .connect_timeout(Duration::from_secs(10))
        .build()?;

    match client.get("https://www.google.com").send().await {
        Ok(resp) if resp.status().is_success() => {
            println!("✅ Healthy: {}", resp.status());
        }
        _ => {
            let msg = "🚨 Proxy Down! Restarting Docker stack...";
            let _ = send_telegram_alert( msg).await;
            restart_docker();
        }
    }
    Ok(())
}

async fn send_telegram_alert(message: &str) -> anyhow::Result<()> {
    let token: String = env::var("BOT_TOKEN")?;
    let chat_id: String = env::var("CHAT_ID")?;
    
    let url: String = format!("https://api.telegram.org/bot{}/sendMessage", token);
    let client: Client = Client::new();
    
    client.post(url)
        .form(&[
            ("chat_id", chat_id.as_str()), 
            ("text", message)
        ])
        .send()
        .await?;
    
    Ok(())
}

fn restart_docker() {
    let compose_path = env::var("COMPOSE_FILE").unwrap_or_else(|_| "docker-compose.yml".to_string());
    let _ = Command::new("sudo")
        .args(["docker", "compose", "-f", &compose_path, "restart"])
        .status();
}