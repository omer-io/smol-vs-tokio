use async_net::TcpListener;
use smol::{block_on, spawn};
// use smol::io::{AsyncReadExt, AsyncWriteExt, BufReader};
use smol::io::{BufReader};
// use smol::io::AsyncBufReadExt;
use futures::prelude::*;
use reqwest::Client;
use async_compat::CompatExt;

async fn get_tvl(
    client: &Client,
    base_url: &str,
    db: &str,
    username: &str,
    password: &str,
    epoch: u64,
) -> Option<f64> {
    let query = format!("SELECT \"tvl\" FROM \"tvl\" WHERE \"epoch\" = {}", epoch);
    let url = format!(
        "{}/query?db={}&q={}",
        base_url,
        db,
        urlencoding::encode(&query)
    );

    match client
        .get(&url)
        .basic_auth(username, Some(password))
        .send()
        .compat()
        .await
    {
        Ok(resp) => {
            let json: serde_json::Value = resp.json().compat().await.unwrap_or_default();
            // println!("json: {:?}", json);
            if let Some(arr) = json["results"][0]["series"][0]["values"].as_array() {
                if let Some(tvl_val) = arr[0][1].as_f64() {
                    return Some((tvl_val / 1e9 * 100.0).round() / 100.0); // round to 2 decimals
                }
            }
            None
        }
        Err(e) => {
            eprintln!("Query error: {}", e);
            None
        }
    }
}

fn main() {
    block_on(async {
        let base_url = "https://api.rakurai.io:8086";
        let db = "mainnet_metrics";
        let username = "mainnet_metrics_read";
        let password = "read";

        let db_client = Client::builder()
        .build()
        .unwrap();

        let listener = TcpListener::bind("127.0.0.1:12345").await.unwrap();
        println!("Smol server running...");

        loop {
            let (mut stream, _) = listener.accept().await.unwrap();

            let db_client = db_client.clone();
            let base_url = base_url.to_string();
            let db = db.to_string();
            let username = username.to_string();
            let password = password.to_string();

            spawn(async move {
                let (reader, mut writer) = stream.split();
                let mut reader = BufReader::new(reader);
                let mut line = String::new();
                loop {
                    line.clear();
                    let n = match reader.read_line(&mut line).await {
                        Ok(0) => break, // connection closed
                        Ok(n) => n,
                        Err(_) => break,
                    };

                    let parts: Vec<&str> = line.trim().split_whitespace().collect();
                    if n > 0 && parts.len() == 4 && parts[0] == "START" && parts[2] == "END" {
                        let start_epoch: u64 = parts[1].parse().unwrap_or(0);
                        let end_epoch: u64 = parts[3].parse().unwrap_or(0);
                        // stream one line per epoch
                        for epoch in start_epoch..=end_epoch {
                            if let Some(tvl) = get_tvl(
                                &db_client,
                                &base_url,
                                &db,
                                &username,
                                &password,
                                epoch,
                            )
                            .await
                            {
                                // keep the original human-readable style
                                let reply = format!("epoch {} => tvl {}\n", epoch, tvl);
                                if writer.write_all(reply.as_bytes()).await.is_err() {
                                    return;
                                }
                            }
                        }
                        // explicit batch terminator
                        if writer.write_all(b"END\n").await.is_err() {
                            return;
                        }
                    }
                }
            }).detach();
        }
    });
}
