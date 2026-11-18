use tracing_subscriber::EnvFilter;

/// Log çıktısını kurar: `RUST_LOG` (varsayılan `info`), `json` ise satır
/// başına bir JSON nesnesi. Zaten kuruluysa bir şey yapmaz.
pub fn init_tracing(json: bool) {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let builder = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(std::io::stderr);
    let _ = if json {
        builder.json().try_init()
    } else {
        builder.try_init()
    };
}
