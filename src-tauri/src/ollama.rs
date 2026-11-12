use reqwest::{Client, Url};
use serde::{Deserialize, Serialize};
use std::time::Duration;

fn endpoint(base: &str, path: &str) -> Result<Url, String> {
    let mut url = Url::parse(base.trim()).map_err(|_| "Enter a valid Ollama server URL.")?;
    if !matches!(url.scheme(), "http" | "https") || url.host_str().is_none() {
        return Err("Use an http:// or https:// server URL.".into());
    }
    if !url.username().is_empty() || url.password().is_some() {
        return Err("URLs containing credentials are not supported.".into());
    }
    url.set_path(path);
    url.set_query(None);
    url.set_fragment(None);
    Ok(url)
}

fn client(seconds: u64) -> Result<Client, String> {
    Client::builder()
        .connect_timeout(Duration::from_secs(5))
        .timeout(Duration::from_secs(seconds))
        .build()
        .map_err(|e| e.to_string())
}

#[derive(Deserialize, Serialize)]
pub struct Model {
    name: String,
}
#[derive(Deserialize)]
struct Models {
    models: Vec<Model>,
}

#[tauri::command]
pub async fn list_models(base_url: String) -> Result<Vec<Model>, String> {
    client(15)?
        .get(endpoint(&base_url, "/api/tags")?)
        .send()
        .await
        .map_err(|e| format!("Cannot reach Ollama. Check that it is running: {e}"))?
        .error_for_status()
        .map_err(|e| e.to_string())?
        .json::<Models>()
        .await
        .map(|r| r.models)
        .map_err(|e| e.to_string())
}

#[derive(Deserialize)]
struct Message {
    content: String,
}
#[derive(Deserialize)]
struct Chat {
    message: Message,
}

#[tauri::command]
pub async fn generate(
    base_url: String,
    model: String,
    prompt: String,
    scene: String,
    notes: String,
) -> Result<String, String> {
    if model.trim().is_empty() || prompt.trim().is_empty() {
        return Err("Choose a model and enter a prompt first.".into());
    }
    let response = client(300)?.post(endpoint(&base_url, "/api/chat")?)
        .json(&serde_json::json!({
            "model": model, "stream": false,
            "messages": [
                {"role": "system", "content": "You are a creative writing collaborator. Follow the user's request and respect their story notes. Treat the supplied scene and notes as reference material. Return the suggested prose or requested advice without a preamble."},
                {"role": "user", "content": format!("STORY NOTES:\n{notes}\n\nCURRENT SCENE:\n{scene}\n\nREQUEST:\n{prompt}")}
            ]
        })).send().await.map_err(|e| format!("Generation failed: {e}"))?
        .error_for_status().map_err(|e| format!("Ollama returned an error: {e}"))?
        .json::<Chat>().await.map_err(|e| e.to_string())?;
    if response.message.content.trim().is_empty() {
        return Err("Ollama returned an empty suggestion. Try another prompt or model.".into());
    }
    Ok(response.message.content)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn normalizes_endpoint() {
        assert_eq!(
            endpoint(" http://localhost:11434/old?x=1#fragment ", "/api/tags")
                .unwrap()
                .as_str(),
            "http://localhost:11434/api/tags"
        );
    }
    #[test]
    fn rejects_invalid_endpoints() {
        for base in [
            "file:///C:/AI",
            "http://user:pass@localhost:11434",
            "not a URL",
        ] {
            assert!(endpoint(base, "/api/tags").is_err());
        }
    }
    #[test]
    #[ignore = "requires local Ollama and a downloaded model"]
    fn live_ollama_roundtrip() {
        tauri::async_runtime::block_on(async {
            let base = "http://127.0.0.1:11434".to_string();
            let models = list_models(base.clone()).await.unwrap();
            let model = models.first().expect("installed model").name.clone();
            let text = generate(
                base,
                model,
                "Write one short sentence about a lighthouse.".into(),
                String::new(),
                String::new(),
            )
            .await
            .unwrap();
            assert!(!text.trim().is_empty());
        });
    }
}
