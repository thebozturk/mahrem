//! Uç adresinin denetimi.

use std::net::IpAddr;

use reqwest::Url;

use crate::LlmError;

/// Taban adresi okur ve denetler; `chat/completions` adresini döner.
///
/// Yalnızca `http` desteklenir. `allow_remote` kapalıyken uç yerel olmalıdır:
/// `localhost`, 127.0.0.0/8 ya da `::1`.
pub(crate) fn chat_url(base_url: &str, allow_remote: bool) -> Result<Url, LlmError> {
    let base = Url::parse(base_url)
        .map_err(|e| LlmError::Config(format!("geçersiz adres {base_url:?}: {e}")))?;
    match base.scheme() {
        "http" => {}
        "https" => {
            return Err(LlmError::Config(
                "https henüz desteklenmiyor; TLS ilk uzak uç ihtiyacıyla gelir".into(),
            ));
        }
        other => return Err(LlmError::Config(format!("desteklenmeyen şema: {other}"))),
    }
    if !base.username().is_empty() || base.password().is_some() {
        return Err(LlmError::Config(
            "adreste kullanıcı bilgisi olmamalı; anahtar `api_key` ile verilir".into(),
        ));
    }
    let host = base.host_str().unwrap_or_default();
    if !allow_remote && !is_local(host) {
        return Err(LlmError::RemoteEndpoint(host.to_owned()));
    }
    let joined = format!("{}/chat/completions", base.as_str().trim_end_matches('/'));
    Url::parse(&joined).map_err(|e| LlmError::Config(format!("geçersiz adres {joined:?}: {e}")))
}

/// Adres yerel bir uca mı işaret ediyor?
pub(crate) fn is_local(host: &str) -> bool {
    if host.eq_ignore_ascii_case("localhost") {
        return true;
    }
    let bare = host.trim_start_matches('[').trim_end_matches(']');
    bare.parse::<IpAddr>().is_ok_and(|ip| ip.is_loopback())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_hosts() {
        for host in ["localhost", "LOCALHOST", "127.0.0.1", "127.0.0.2", "[::1]"] {
            assert!(is_local(host), "{host}");
        }
        for host in [
            "",
            "0.0.0.0",
            "10.0.0.5",
            "192.168.1.10",
            "example.com",
            "localhost.example.com",
            "[::]",
        ] {
            assert!(!is_local(host), "{host}");
        }
    }

    #[test]
    fn url_is_joined() {
        let url = chat_url("http://127.0.0.1:8080/v1/", false).unwrap();
        assert_eq!(url.as_str(), "http://127.0.0.1:8080/v1/chat/completions");
        let url = chat_url("http://localhost:11434/v1", false).unwrap();
        assert_eq!(url.as_str(), "http://localhost:11434/v1/chat/completions");
    }

    #[test]
    fn remote_needs_permission() {
        assert!(matches!(
            chat_url("http://10.0.0.5/v1", false),
            Err(LlmError::RemoteEndpoint(_))
        ));
        assert!(chat_url("http://10.0.0.5/v1", true).is_ok());
    }

    #[test]
    fn bad_urls() {
        for url in [
            "https://127.0.0.1/v1",
            "ftp://127.0.0.1/v1",
            "127.0.0.1:8080",
            "http://user:pw@127.0.0.1/v1",
        ] {
            assert!(
                matches!(chat_url(url, true), Err(LlmError::Config(_))),
                "{url}"
            );
        }
    }
}
