use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{self, BufRead, BufReader};

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
async fn validate_word(
    query: web::Query<std::collections::HashMap<String, String>>,
) -> impl Responder {
    let word = match query.get("word") {
        Some(w) => w.to_lowercase(),
        None => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Missing 'word' query parameter"
            }));
        }
    };

    // Check local dictionary file line-by-line
    if let Ok(found) = word_in_dictionary(&word) {
        if found {
            return HttpResponse::Ok().json(ValidationResponse {
                word: word.clone(),
                found: true,
                source: "local".to_string(),
            });
        }
    }

    // If not found locally, check external APIs
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

fn word_in_dictionary(word: &str) -> io::Result<bool> {
    let file = File::open("dictionary.txt")?;
    let reader = BufReader::new(file);

    for line in reader.lines() {
        if let Ok(line) = line {
            let clean = line
                .chars()
                .filter(|c| c.is_alphanumeric())
                .collect::<String>()
                .to_lowercase();
            if clean == word {
                return Ok(true);
            }
        }
    }
    Ok(false)
}

async fn check_external_apis(word: &str) -> Result<bool, Box<dyn std::error::Error>> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()?;

    // Free Dictionary API
    let free_dict_url = format!("https://api.dictionaryapi.dev/api/v2/entries/en/{}", word);
    if let Ok(response) = client.get(&free_dict_url).send().await {
        if response.status().is_success() {
            return Ok(true);
        }
    }

    // Datamuse API
    let datamuse_url = format!("https://api.datamuse.com/words?sp={}&max=1", word);
    if let Ok(response) = client.get(&datamuse_url).send().await {
        if let Ok(data) = response.json::<Vec<DatamuseResponse>>().await {
            if !data.is_empty() && data[0].word.to_lowercase() == word {
                return Ok(true);
            }
        }
    }

    // Words API (Wordnik)
    let words_api_url = format!(
        "https://api.wordnik.com/v4/word.json/{}/definitions?api_key=test",
        word
    );
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
        "version": "0.1.1",
        "usage": "/word?word=<word_to_validate>",
        "dictionary_load": "on-demand"
    }))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("📚 Word Validator API (streaming dictionary)");
    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    println!("🚀 Starting server on http://0.0.0.0:{}", port);

    HttpServer::new(|| App::new().service(index).service(validate_word))
        .bind(("0.0.0.0", port.parse().unwrap()))?
        .run()
        .await
}
