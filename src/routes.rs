use actix_web::{get, web, HttpResponse, Responder};
use log::info;

use crate::appstate::{cache_image, get_cached_image, get_product};
//http://192.168.178.138:8080/thumbnail_international/20005733/en/
#[get("/thumbnail_international/{gtin}/{language}/")]
pub async fn get_international_thumb(path: web::Path<(String, String)>) -> impl Responder {
    let (gtin, language) = path.into_inner();
    let product = get_product(gtin).unwrap();

    match product {
        Some(product) => {
            let link = product.get_front_link(Some(language), 100);
            let cached = get_cached_image(&link).await;
            if let Ok(Some(on_disk)) = cached {
                info!("Image {} is on disk!", product.id);
                return HttpResponse::Ok().content_type("image/jpg").body(on_disk);
            }
            tokio::spawn({
                let link = link.clone();
                async move {
                    cache_image(&link).await.unwrap();
                    info!("Cached {}", product.id);
                }
            });
            //println!("Cached {}", product.id);

            HttpResponse::Found()
                .append_header(("Location", link))
                .finish()
        }
        None => HttpResponse::NotFound().finish(),
    }
}

#[get("/thumbnail/{gtin}/")]
pub async fn get_product_thumb(path: web::Path<String>) -> impl Responder {
    let gtin = path.into_inner();
    let product = get_product(gtin).unwrap();

    match product {
        Some(product) => HttpResponse::Found()
            .append_header(("Location", product.get_front_link(None, 100)))
            .finish(),
        None => HttpResponse::NotFound().finish(),
    }
}

#[get("/front/{gtin}/")]
pub async fn get_product_400(path: web::Path<String>) -> impl Responder {
    let gtin = path.into_inner();
    let product = get_product(gtin).unwrap();

    match product {
        Some(product) => HttpResponse::Found()
            .append_header(("Location", product.get_front_link(None, 400)))
            .finish(),
        None => HttpResponse::NotFound().finish(),
    }
}
