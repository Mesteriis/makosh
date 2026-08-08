use std::env;

use makosh_hub_backend::integrations::ollama::client::{OllamaClient, config::OllamaClientConfig};

#[tokio::test]
async fn live_ollama_qwen3_runtime_smoke() {
    let Some(base_url) = env::var("MAKOSH_OLLAMA_BASE_URL").ok() else {
        eprintln!("skipping live Ollama smoke test: MAKOSH_OLLAMA_BASE_URL is not set");
        return;
    };
    let chat_model = env::var("MAKOSH_OLLAMA_CHAT_MODEL").unwrap_or_else(|_| "qwen3:4b".to_owned());
    let embed_model =
        env::var("MAKOSH_OLLAMA_EMBED_MODEL").unwrap_or_else(|_| "qwen3-embedding:4b".to_owned());
    let timeout_seconds = env::var("MAKOSH_OLLAMA_TIMEOUT_SECONDS")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(120);

    let client = OllamaClient::new(
        OllamaClientConfig::new(base_url, chat_model.clone(), embed_model.clone())
            .with_timeout_seconds(timeout_seconds),
    )
    .expect("Ollama client");

    let version = client.version().await.expect("Ollama version");
    assert!(!version.trim().is_empty());

    let tags = client.tags().await.expect("Ollama tags");
    assert!(
        tags.iter().any(|model| model == &chat_model),
        "missing chat model {chat_model}; available models: {tags:?}"
    );
    assert!(
        tags.iter().any(|model| model == &embed_model),
        "missing embedding model {embed_model}; available models: {tags:?}"
    );

    let chat = client
        .chat("Return exactly this token and nothing else: makosh-ai-smoke-ok")
        .await
        .expect("Ollama chat");
    assert!(
        chat.content.contains("makosh-ai-smoke-ok"),
        "unexpected chat response: {}",
        chat.content
    );

    let embedding = client
        .embed("Макошь V3 AI semantic retrieval smoke")
        .await
        .expect("Ollama embed");
    assert_eq!(embedding.embedding.len(), 2560);
}
