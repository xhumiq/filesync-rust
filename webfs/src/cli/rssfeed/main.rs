use anyhow::Context;
use std::result::Result;
use chrono::{Utc};
use clap::{Arg, Command};
use quick_xml::Writer;
use std::fs::File;
use std::io::BufWriter;
use webfs::models::files::{Config, Channel};

fn default_filter_extension() -> String {
    "".to_string()
}

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    dotenv::dotenv().ok();
    webfs::init_tracing("../logs/rssfeed.log")?;
    tracing::info!("Application started");

    let matches = Command::new("rss_writer")
        .arg(Arg::new("channel")
            .long("channel")
            .value_name("CHANNEL_NAME")
            .help("The channel name")
            .required(true))
        .arg(Arg::new("config")
            .long("config")
            .value_name("CONFIG_FILE")
            .help("Path to config file")
            .default_value("config.yaml"))
        .arg(Arg::new("language")
            .long("lang")
            .value_name("LANGUAGE")
            .help("Language code")
            .default_value("en"))
        .arg(Arg::new("days")
            .long("days")
            .value_name("DAYS")
            .help("Number of days of history to include in the RSS feed"))
        .arg(Arg::new("log_file")
            .long("log-file")
            .value_name("LOG_FILE")
            .help("Path to log file")
            .default_value("/opt/webdav/logs/rssfeed/rssfeed.log"))
        .get_matches();

    let log_file = matches.get_one::<String>("log_file").ok_or("log_file argument missing")?;
    let channel_arg = matches.get_one::<String>("channel").ok_or("channel argument missing")?;
    let config_path = matches.get_one::<String>("config").ok_or("config argument missing")?;
    let language = matches.get_one::<String>("language").ok_or("language argument missing")?;
    let start_date = matches.get_one::<String>("days")
        .and_then(|s| s.parse::<i32>().ok())
        .map(|days| Utc::now().date_naive() - chrono::Duration::days(days.abs() as i64));

    let config: Config = Channel::read_config(&config_path)?;

    let lang_channels = config.channels.get(language)
        .ok_or_else(|| anyhow::anyhow!("Language '{}' not found", language))?;

    let channels_to_process: Vec<(&String, &Channel)> = if channel_arg == "all" {
        lang_channels.iter().collect()
    } else {
        match lang_channels.get_key_value(channel_arg) {
            Some(kv) => vec![kv],
            None => {
                tracing::error!("Channel '{}' not found in language '{}'. Available channels:", channel_arg, language);
                for (name, ch) in lang_channels.iter() {
                    tracing::error!("  {} -> {}", name, ch.output_path);
                }
                return Ok(());
            }
        }
    };

    if channels_to_process.is_empty() {
        tracing::info!("No channels found in config file");
        return Ok(());
    }
    // if there are more then one channel to process - print the number of them
    tracing::info!("Processing {} channels", channels_to_process.len());
    for (channel_name, ch) in channels_to_process {

        let output_path = &ch.output_path;

        tracing::info!("---------------------------------------------------------");
        tracing::info!("Refreshing RSS Channel {} {}", channel_name, output_path);

        // Read and filter files from the directory
        let entries = Channel::read_dir(&ch)?;
        if entries.is_empty() {
            tracing::warn!("No entries found for channel {}", channel_name);
            continue;
        }

        // Create output file and XML writer
        let file = File::create(output_path).context("Failed to create output file")?;
        let buf_writer = BufWriter::new(file);
        let mut writer = Writer::new(buf_writer);

        // Process entries
        let mut ch = ch.clone();
        ch.set_entries(entries);

        // Write RSS
        ch.write_rss(&mut writer, start_date)?;

        // Print first ten file names of sorted entries in channel
        for (i, entry) in ch.entries.iter().take(10).enumerate() {
            tracing::info!("{}: {} {}", i + 1, entry.file_name, entry.location);
        }

        tracing::info!("RSS feed written to {} with {} entries", output_path, ch.entries.len());
    }

    Ok(())
}