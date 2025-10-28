use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;

// Global dictionary loaded once at startup
static DICTIONARY: Lazy<HashSet<String>> = Lazy::new(|| {
    let content = fs::read_to_string("dictionary.txt")
        .unwrap_or_else(|_| String::new());
    
    // Split by whitespace, strip symbols, keep only fully alphanumeric words
    content
        .split_whitespace()
        .map(|word| {
            // Strip non-alphanumeric characters and convert to lowercase
            word.chars()
                .filter(|c| c.is_alphanumeric())
                .collect::<String>()
                .to_lowercase()
        })
        .filter(|word| !word.is_empty() && word.chars().all(|c| c.is_alphanumeric()))
        .collect()
});

#[derive(Serialize)]
struct ValidationResponse {
    word: String,
    found: bool,
    source: String,
}

#[derive(Deserialize)]
struct DatamuseResponse {
    word: String,
}

#[get("/word")]
async fn validate_word(query: web::Query<std::collections::HashMap<String, String>>) -> impl Responder {
    let word = match query.get("word") {
        Some(w) => w.to_lowercase(),
        None => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Missing 'word' query parameter"
            }));
        }
    };

    // Check local dictionary first
    if DICTIONARY.contains(&word) {
        return HttpResponse::Ok().json(ValidationResponse {
            word: word.clone(),
            found: true,
            source: "local".to_string(),
        });
    }

    // If not found locally, query external APIs
    if let Ok(found) = check_external_apis(&word).await {
        if found {
            return HttpResponse::Ok().json(ValidationResponse {
                word: word.clone(),
                found: true,
                source: "external".to_string(),
            });
        }
    }

    HttpResponse::Ok().json(ValidationResponse {
        word,
        found: false,
        source: "none".to_string(),
    })
}

async fn check_external_apis(word: &str) -> Result<bool, Box<dyn std::error::Error>> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()?;

    // 1. Check Free Dictionary API
    let free_dict_url = format!("https://api.dictionaryapi.dev/api/v2/entries/en/{}", word);
    if let Ok(response) = client.get(&free_dict_url).send().await {
        if response.status().is_success() {
            return Ok(true);
        }
    }

    // 2. Check Datamuse API
    let datamuse_url = format!("https://api.datamuse.com/words?sp={}&max=1", word);
    if let Ok(response) = client.get(&datamuse_url).send().await {
        if let Ok(data) = response.json::<Vec<DatamuseResponse>>().await {
            if !data.is_empty() && data[0].word.to_lowercase() == word.to_lowercase() {
                return Ok(true);
            }
        }
    }

    // 3. Check Words API (wordsapi.com - free tier available)
    let words_api_url = format!("https://api.wordnik.com/v4/word.json/{}/definitions?api_key=test", word);
    if let Ok(response) = client.get(&words_api_url).send().await {
        if response.status().is_success() {
            return Ok(true);
        }
    }

    Ok(false)
}

#[get("/")]
async fn index() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "service": "Word Validator API",
        "version": "0.1.0",
        "usage": "/word?word=<word_to_validate>",
        "dictionary_size": DICTIONARY.len()
    }))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("📚 Word Validator API");
    println!("📖 Loaded {} words from dictionary", DICTIONARY.len());
    println!("🚀 Starting server on http://0.0.0.0:8080");

    HttpServer::new(|| {
        App::new()
            .service(index)
            .service(validate_word)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
