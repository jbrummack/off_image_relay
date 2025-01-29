use std::{
    collections::HashMap,
    thread,
    time::{Instant, SystemTime, UNIX_EPOCH},
};

use cjstream::extract_json::{bounded, stream_jsonl};

use log::info;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::appstate::index_product;

#[derive(Debug, Deserialize, Serialize)]
pub struct ProductWithImages {
    pub id: String,
    pub images: HashMap<String, ImageInfo>,
}

pub async fn should_index(days_interval: u64) -> bool {
    let _ = tokio::fs::create_dir_all("idx_log").await;
    let interval_as_s = 60 * 60 * 24 * days_interval;
    if let Ok(mut dirs) = tokio::fs::read_dir("idx_log").await {
        let mut last_ts = 0;
        let begin = UNIX_EPOCH;
        let now = SystemTime::now()
            .duration_since(begin)
            .map_err(|e| format!("Time error {e}"));
        let now = match now {
            Ok(ok) => ok.as_secs(),
            Err(e) => {
                log::error!("{e}");
                return false;
            }
        };
        while let Ok(Some(dir)) = dirs.next_entry().await {
            let ts = dir
                .file_name()
                .into_string()
                .map_err(|_| format!("Couldnt convert filename of {dir:?} to String!"))
                .and_then(|s| {
                    s.parse::<u64>()
                        .map_err(|e| format!("Couldnt parse filename to timestamp ({e})"))
                });
            match ts {
                Ok(ts) => {
                    if ts > last_ts {
                        last_ts = ts;
                    }
                }
                Err(e) => log::error!("Failed to get last timestamp: {e}"),
            }
        }
        if (now - last_ts) > interval_as_s {
            let _ = tokio::fs::File::create(format!("idx_log/{now}")).await;
            return true;
        }
    }
    false
}

pub fn create_image_link(gtin: u64, resolution: u16, imgid: u16) -> String {
    let mut folders = format!("{gtin}");
    loop {
        if folders.len() > 12 {
            break;
        }
        //println!("{folders}");
        folders = format!("0{folders}");
    }
    let end = folders.split_off(9);
    let n3 = folders.split_off(6);
    let n2 = folders.split_off(3);
    format!("https://images.openfoodfacts.org/images/products/{folders}/{n2}/{n3}/{end}/{imgid}.{resolution}.jpg")
}

impl ProductWithImages {
    pub fn get_front_link(&self, preferred_language: Option<String>, resolution: u16) -> String {
        let front = self
            .get_front(preferred_language)
            .as_str()
            .replace("\"", "")
            .replace(".0", "")
            .parse::<u16>();
        let id = self.id.parse::<u64>();
        if let (Ok(gtin), Ok(front)) = (&id, front.clone()) {
            create_image_link(*gtin, resolution, front)
        } else {
            format!("Error: at {:?}", (id, front))
        }
    }

    pub fn get_front(&self, preferred_language: Option<String>) -> String {
        if let Some(pref) = preferred_language {
            let look_for = format!("front_{pref}");
            if let Some(id) = self.images.get(&look_for) {
                if let Some(imgid) = &id.imgid {
                    let id = imgid.to_string();
                    return id;
                }
            }
        };
        let look_for = format!("front");
        if let Some(id) = self.images.get(&look_for) {
            if let Some(imgid) = &id.imgid {
                let id = imgid.to_string();
                return id;
            }
        }
        let look_for = format!("front_en");
        if let Some(id) = self.images.get(&look_for) {
            if let Some(imgid) = &id.imgid {
                let id = imgid.to_string();
                return id;
            }
        }

        for (key, value) in &self.images {
            if key.as_str().contains("front") {
                if let Some(id) = &value.imgid {
                    return id.to_string();
                }
            }
        }

        String::from("1")
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ImageInfo {
    uploaded_t: Option<Value>,
    imgid: Option<Value>,
}
/*
pub struct AppState {}

impl AppState {
    fn get_image() {}
    fn put_image(gtin: String) {}
} */

pub fn index(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    //std::fs::create_dir_all("state")?;

    info!("Indexing {path}");
    let now = Instant::now();
    let (tx, rx) = bounded(64);
    let producer = stream_jsonl(path, tx)?;

    let consumer = thread::spawn({
        move || {
            let mut amnt = 0;

            while let Ok(rx) = rx.recv() {
                amnt += 1;
                if let Ok(value) = serde_json::from_str::<ProductWithImages>(rx.as_str()) {
                    //println!("{value:#?}");
                    index_product(value);
                }
            }
            println!("{amnt}");
        }
    });
    producer.join().unwrap();
    consumer.join().unwrap();
    let elapsed_time = now.elapsed();
    info!("Indexing took {} seconds.", elapsed_time.as_secs());
    Ok(())
}
