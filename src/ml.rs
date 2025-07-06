use actix_web::{post, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use reqwest::Client;
use urlencoding::encode;

/// Incoming JSON shape
#[derive(Deserialize)]
pub struct TranslateReq {
    pub q: String,
    #[serde(default)]
    pub source: String,   // ignored by MyMemory
    pub target: String,
}

/// Outgoing JSON shape
#[derive(Serialize)]
struct MMMatch {
    /// translated text
    translatedText: String,
}

#[derive(Deserialize)]
struct MMResp {
    responseData: ResponseData,
}

#[derive(Deserialize)]
struct ResponseData {
    translatedText: String,
}

#[post("/translate")]
pub async fn translate(body: web::Json<TranslateReq>) -> impl Responder {
    let client = Client::new();
    // URL‐encode q and build MyMemory URL
    let q_enc = encode(&body.q);
    let pair = format!("{}|{}", "en", &body.target); // we’ll assume source “en” if empty
    let url = format!("https://api.mymemory.translated.net/get?q={}&langpair={}", q_enc, pair);

    match client.get(&url).send().await {
        Ok(resp) => match resp.json::<MMResp>().await {
            Ok(mm) => HttpResponse::Ok().json(MMMatch {
                translatedText: mm.responseData.translatedText,
            }),
            Err(_) => HttpResponse::BadGateway().body("Parse error from translator"),
        },
        Err(_) => HttpResponse::BadGateway().body("Translator unreachable"),
    }
}
