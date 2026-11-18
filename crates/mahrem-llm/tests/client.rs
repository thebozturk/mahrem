//! Sahte bir sunucuya karşı istemci davranışı: yeniden deneme, hata sınıfları,
//! eşzamanlılık ve yerel uç koruması.

use std::time::{Duration, Instant};

use mahrem_llm::{ChatRequest, Client, ClientConfig, LlmError, Message, Usage};
use serde_json::json;
use wiremock::matchers::{body_partial_json, header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn ok_body(content: &str) -> serde_json::Value {
    json!({
        "model": "test-model",
        "choices": [{"index": 0, "message": {"role": "assistant", "content": content}, "finish_reason": "stop"}],
        "usage": {"prompt_tokens": 12, "completion_tokens": 5, "total_tokens": 17}
    })
}

fn config(server: &MockServer) -> ClientConfig {
    let mut c = ClientConfig::local(format!("{}/v1", server.uri()));
    c.backoff_base = Duration::from_millis(5);
    c.backoff_max = Duration::from_secs(5);
    c.max_retries = 3;
    c.timeout = Duration::from_secs(5);
    c
}

fn request() -> ChatRequest {
    ChatRequest::new("test-model", vec![Message::user("Kargom nerede?")])
        .seed(42)
        .temperature(0.7)
}

async fn received(server: &MockServer) -> usize {
    server.received_requests().await.unwrap().len()
}

#[tokio::test]
async fn success_and_usage() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .and(header("authorization", "Bearer gizli"))
        .and(body_partial_json(
            json!({"model": "test-model", "seed": 42, "stream": false}),
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(ok_body("Yolda.")))
        .mount(&server)
        .await;
    let mut c = config(&server);
    c.api_key = Some("gizli".into());
    let client = Client::new(c).unwrap();
    for _ in 0..2 {
        let resp = client.chat(&request()).await.unwrap();
        assert_eq!(resp.content, "Yolda.");
        assert_eq!(resp.finish_reason.as_deref(), Some("stop"));
        assert_eq!(resp.model.as_deref(), Some("test-model"));
    }
    assert_eq!(
        client.usage(),
        Usage {
            prompt_tokens: 24,
            completion_tokens: 10
        }
    );
}

#[tokio::test]
async fn server_errors_are_retried() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(503).set_body_string("meşgul"))
        .up_to_n_times(2)
        .with_priority(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_json(ok_body("tamam")))
        .mount(&server)
        .await;
    let client = Client::new(config(&server)).unwrap();
    assert_eq!(client.chat(&request()).await.unwrap().content, "tamam");
    assert_eq!(received(&server).await, 3);
}

#[tokio::test]
async fn retries_run_out() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(500).set_body_string("çöktü"))
        .mount(&server)
        .await;
    let mut c = config(&server);
    c.max_retries = 2;
    let client = Client::new(c).unwrap();
    match client.chat(&request()).await {
        Err(LlmError::Exhausted { attempts, last }) => {
            assert_eq!(attempts, 3);
            assert!(matches!(*last, LlmError::Server { status: 500, ref body } if body == "çöktü"));
        }
        other => panic!("{other:?}"),
    }
    assert_eq!(received(&server).await, 3);
    assert_eq!(client.usage(), Usage::default());
}

#[tokio::test]
async fn rate_limit_honors_retry_after() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(429).insert_header("retry-after", "1"))
        .up_to_n_times(1)
        .with_priority(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_json(ok_body("sonunda")))
        .mount(&server)
        .await;
    let client = Client::new(config(&server)).unwrap();
    let start = Instant::now();
    assert_eq!(client.chat(&request()).await.unwrap().content, "sonunda");
    assert!(
        start.elapsed() >= Duration::from_secs(1),
        "{:?}",
        start.elapsed()
    );
    assert_eq!(received(&server).await, 2);
}

#[tokio::test]
async fn too_long_retry_after_is_not_waited() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(429).insert_header("retry-after", "3600"))
        .mount(&server)
        .await;
    let client = Client::new(config(&server)).unwrap();
    let start = Instant::now();
    let err = client.chat(&request()).await.unwrap_err();
    assert!(matches!(
        err,
        LlmError::RateLimited { retry_after: Some(d) } if d == Duration::from_secs(3600)
    ));
    assert!(start.elapsed() < Duration::from_secs(1));
    assert_eq!(received(&server).await, 1);
}

#[tokio::test]
async fn client_errors_are_not_retried() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(400).set_body_string("model yok"))
        .mount(&server)
        .await;
    let client = Client::new(config(&server)).unwrap();
    let err = client.chat(&request()).await.unwrap_err();
    assert!(matches!(err, LlmError::Rejected { status: 400, ref body } if body == "model yok"));
    assert!(!err.is_retryable());
    assert_eq!(received(&server).await, 1);
}

#[tokio::test]
async fn timeout_is_retried_then_reported() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(ok_body("geç"))
                .set_delay(Duration::from_secs(2)),
        )
        .mount(&server)
        .await;
    let mut c = config(&server);
    c.timeout = Duration::from_millis(200);
    c.max_retries = 1;
    let client = Client::new(c).unwrap();
    match client.chat(&request()).await {
        Err(LlmError::Exhausted { attempts: 2, last }) => {
            assert!(matches!(*last, LlmError::Timeout));
        }
        other => panic!("{other:?}"),
    }
}

#[tokio::test]
async fn malformed_responses() {
    for body in [
        ResponseTemplate::new(200).set_body_string("{bozuk"),
        ResponseTemplate::new(200).set_body_json(json!({"choices": []})),
        ResponseTemplate::new(200)
            .set_body_json(json!({"choices": [{"message": {"role": "assistant"}}]})),
        ResponseTemplate::new(200).set_body_json(json!({"sonuç": "yok"})),
    ] {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .respond_with(body)
            .mount(&server)
            .await;
        let client = Client::new(config(&server)).unwrap();
        let err = client.chat(&request()).await.unwrap_err();
        assert!(matches!(err, LlmError::Malformed(_)), "{err:?}");
        assert_eq!(received(&server).await, 1);
    }
}

#[tokio::test]
async fn connection_refused_is_transport_error() {
    // Boş bir port: dinleyici kapatılınca bağlantı reddedilir.
    let port = std::net::TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port();
    let mut c = ClientConfig::local(format!("http://127.0.0.1:{port}/v1"));
    c.backoff_base = Duration::from_millis(5);
    c.max_retries = 1;
    let client = Client::new(c).unwrap();
    match client.chat(&request()).await {
        Err(LlmError::Exhausted { attempts: 2, last }) => {
            assert!(matches!(*last, LlmError::Transport(_)), "{last:?}");
        }
        other => panic!("{other:?}"),
    }
}

#[tokio::test]
async fn redirects_are_not_followed() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(307).insert_header("location", "http://example.com/v1"))
        .mount(&server)
        .await;
    let client = Client::new(config(&server)).unwrap();
    let err = client.chat(&request()).await.unwrap_err();
    assert!(
        matches!(err, LlmError::Rejected { status: 307, .. }),
        "{err:?}"
    );
}

#[tokio::test]
async fn concurrency_is_limited() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(ok_body("x"))
                .set_delay(Duration::from_millis(300)),
        )
        .mount(&server)
        .await;
    let mut c = config(&server);
    c.concurrency = 1;
    let client = Client::new(c).unwrap();
    let start = Instant::now();
    let req = request();
    let (a, b) = tokio::join!(client.chat(&req), client.chat(&req));
    a.unwrap();
    b.unwrap();
    assert!(
        start.elapsed() >= Duration::from_millis(600),
        "{:?}",
        start.elapsed()
    );
}

#[test]
fn remote_endpoint_needs_permission() {
    let err = Client::new(ClientConfig::local("http://10.1.2.3:8000/v1")).unwrap_err();
    assert!(matches!(err, LlmError::RemoteEndpoint(ref h) if h == "10.1.2.3"));
    let mut c = ClientConfig::local("http://10.1.2.3:8000/v1");
    c.allow_remote = true;
    let client = Client::new(c).unwrap();
    assert!(client.allows_remote());
    assert_eq!(
        client.endpoint(),
        "http://10.1.2.3:8000/v1/chat/completions"
    );
    let mut c = ClientConfig::local("http://127.0.0.1/v1");
    c.concurrency = 0;
    assert!(matches!(Client::new(c), Err(LlmError::Config(_))));
}
