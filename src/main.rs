use std::time::{Duration, Instant};

use actix_web::{middleware::Logger, App, HttpServer};
use appstate::init_app_state;
use indexing::{index, should_index};
use log::{error, info};
//use indexing::index;
use routes::{get_international_thumb, get_product_400, get_product_thumb};
use tokio::{io::AsyncWriteExt, time::interval};

mod appstate;
mod indexing;
mod routes;

// Unable to find libclang: "couldn't find any valid shared libraries matching: ['libclang.so', 'libclang-*.so', 'libclang.so.*', 'libclang-*.so.*'], set the `LIBCLANG_PATH` environment variable to a path where one of t
//i  #10 469.9   note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
#[allow(dead_code)]
pub struct AppConfig {
    ssd_cache_ttl: usize,
    thumbnails_hot: bool,
    fullres_hot: bool,
    s400_hot: bool,
    download_url: String,
}

async fn serve(port: u16) -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    use local_ip_address::local_ip;
    let ip = local_ip().unwrap();
    println!("Serving at {ip}:{port}");
    HttpServer::new(|| {
        App::new()
            .wrap(Logger::default())
            .service(get_international_thumb)
            .service(get_product_thumb)
            .service(get_product_400)
    })
    .bind((ip, port))?
    .run()
    .await?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_app_state();
    tokio::spawn(async {
        let mut interval = interval(Duration::from_secs(60 * 60)); // check every hour
        loop {
            interval.tick().await; // Waits for the next tick
            if should_index(14).await {
                tokio::spawn(async {
                    retrieve_dataset().await.unwrap();
                    index("dataset").unwrap(); //Users/juliusbrummack/Documents/off_json.gz
                    let _ = tokio::fs::remove_file("dataset").await;
                });
            }
        }
    });

    serve(8080).await?;

    Ok(())
}

async fn retrieve_dataset() -> Result<(), Box<dyn std::error::Error>> {
    use futures_util::StreamExt;
    let url = "https://static.openfoodfacts.org/data/openfoodfacts-products.jsonl.gz";

    info!("Downloading {url}");
    let now = Instant::now();

    let response = reqwest::get(url).await?;

    if response.status().is_success() {
        // Open a file to save the downloaded content
        let mut dest = tokio::fs::File::create("dataset").await?;
        let mut stream = response.bytes_stream();
        while let Some(item) = stream.next().await {
            //println!("Chunk: {:?}", item?);
            dest.write_all(&item?).await?;
        }
        info!("File downloaded successfully as: {}", "dataset");
    } else {
        error!("Failed to download file. Status: {}", response.status());
    }
    info!("Download took {}s", now.elapsed().as_secs());

    Ok(())
}
