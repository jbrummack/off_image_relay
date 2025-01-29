use std::sync::OnceLock;

use log::info;
use rocksdb::{Options, DB};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::indexing::ProductWithImages;

pub static APP_STATE: OnceLock<DB> = OnceLock::new();

pub fn state() -> &'static DB {
    APP_STATE
        .get()
        .expect("Please call init_app_state() before using this function!")
}

pub fn index_product(value: ProductWithImages) {
    let payload = serde_json::to_vec(&value).unwrap(); //serde_json::serialize(&value).unwrap();
    state().put(value.id.as_bytes(), payload).unwrap();
}

pub async fn get_cached_image(
    image_url: &str,
) -> Result<Option<Vec<u8>>, Box<dyn std::error::Error>> {
    use blake2::{Blake2s256, Digest};
    // Create a Blake2b hasher
    let mut hasher = Blake2s256::new();

    hasher.update(image_url);
    let result = hasher.finalize();
    let hash_vec: Vec<u8> = result.to_vec();
    let buf = state().get(hash_vec)?;
    //let hash_hex = hex::encode(result);

    //let mut dest = tokio::fs::File::open(format!("img_cache/{hash_hex}.jpg")).await?;
    //let mut buf = Vec::<u8>::new();
    //dest.read_to_end(&mut buf).await?;

    //Ok(Some(buf))
    Ok(buf)
}
pub async fn cache_image(
    image_url: &str,
    //image: image::DynamicImage,
) -> Result<(), Box<dyn std::error::Error>> {
    use blake2::{Blake2s256, Digest};
    // Create a Blake2b hasher
    let mut hasher = Blake2s256::new();
    let response = reqwest::get(image_url).await?;
    if response.status().is_success() {
        let image_data = response.bytes().await?;

        hasher.update(image_url);
        let result = hasher.finalize();

        let mut value = Vec::<u8>::new();
        for byte in image_data {
            value.push(byte);
        }
        let hash_vec: Vec<u8> = result.to_vec();
        state().put(hash_vec, value)?;

        //println!("cached {image_url}")
        //let hash_hex = hex::encode(result);

        //let mut dest = tokio::fs::File::create(format!("img_cache/{hash_hex}.jpg")).await?;
        //dest.write_all(&image_data).await?;
    }
    // Hash the URL

    //let strg = format!("{result:?}");
    //println!("{strg}");
    //image.save(format!("img_cache/{hash_hex}.jpg"))?;
    // Convert the hash to a hex string
    //
    Ok(())
}

pub fn get_product(gtin: String) -> Result<Option<ProductWithImages>, Box<dyn std::error::Error>> {
    let key = gtin.as_bytes();
    let payload = state().get(key)?;
    //.map(|r| bincode::deserialize(&r).unwrap());
    match payload {
        Some(pld) => {
            let res = serde_json::from_slice(pld.as_slice()); //deserialize(pld.as_slice());
                                                              //println!("{res:?}");
            if let Ok(Some(ProductWithImages { id, images })) = &res {
                info!("Got {id}")
            }
            Ok(res?)
        }
        None => Ok(None),
    }
    //let payload = payload.map(|r| bincode::deserialize(r.as_slice()).unwrap());

    //Ok(payload)
}

pub fn init_app_state() {
    let mut opts = Options::default();
    opts.create_if_missing(true);
    opts.set_compression_type(rocksdb::DBCompressionType::Zstd);
    let db = rocksdb::DB::open(&opts, "state").unwrap();
    APP_STATE.set(db).expect("couldnt set appstate");
}
