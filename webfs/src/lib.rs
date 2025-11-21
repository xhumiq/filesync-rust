pub mod auth;
pub mod models;
pub mod webfs;

use reqwest::Client;
use std::io::{BufWriter, Write};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use tracing_subscriber::{fmt, prelude::*, EnvFilter, fmt::MakeWriter};

#[derive(Clone)]
pub struct AppState {
    pub keycloak_url: String,
    pub realm: String,
    pub client_id: String,
    pub client_secret: String,
    pub base_path: String,
    pub http_client: Client,
    pub config: models::files::Config,
    pub channel_cache: Arc<Mutex<HashMap<String, (models::files::Channel, DateTime<Utc>)>>>,
}

pub fn init_tracing(log_path: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Create the parent directory if it doesn't exist
    if let Some(parent) = Path::new(log_path).parent() {
        std::fs::create_dir_all(parent)?;
    }

    let log_file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .append(true)
        .open(log_path)?;

    let buf_writer = BufWriter::new(log_file);

    struct FlushingWriter(BufWriter<std::fs::File>);
    impl Write for FlushingWriter {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            let res = self.0.write(buf);
            self.0.flush()?;
            self.0.get_ref().sync_all()?;
            res
        }
        fn flush(&mut self) -> std::io::Result<()> {
            self.0.flush()?;
            self.0.get_ref().sync_all()
        }
    }

    struct MutexGuardWriter<'a>(std::sync::MutexGuard<'a, FlushingWriter>);
    impl<'a> Write for MutexGuardWriter<'a> {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            self.0.write(buf)
        }
        fn flush(&mut self) -> std::io::Result<()> {
            self.0.flush()
        }
    }

    struct SharedFlushingWriter(Arc<Mutex<FlushingWriter>>);
    impl<'a> MakeWriter<'a> for SharedFlushingWriter {
        type Writer = MutexGuardWriter<'a>;
        fn make_writer(&'a self) -> Self::Writer {
            MutexGuardWriter(self.0.lock().expect("Mutex poisoned"))
        }
    }

    let flushing_writer = FlushingWriter(buf_writer);
    let shared_writer = SharedFlushingWriter(Arc::new(Mutex::new(flushing_writer)));

    // Set up subscriber: log to both file and stdout with compact format
    tracing_subscriber::registry()
        .with(EnvFilter::from_default_env()
            .add_directive("rssfeed=debug".parse()?)
            .add_directive("webfs=debug".parse()?)) // Respect RUST_LOG env var, default to info for webfs
        .with(
            fmt::layer()
                .with_writer(shared_writer)
                .with_ansi(false)
                .compact()
        )
        .with(
            fmt::layer()
                .compact()
        )
        .init();

    Ok(())
}

