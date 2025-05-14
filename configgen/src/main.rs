use clap::Parser;
use qrcode::render::unicode;
use qrcode::render::unicode::Dense1x2;
use qrcode::QrCode;
use regex::Regex;
use serde::Deserialize;
use serde_json;
use serde_yaml;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::Write;
use url::Url;
use urlencoding;

#[derive(Debug, Deserialize)]
struct Client {
    protocol: String,
    id: String,
    address: String,
    port: u16,
    params: HashMap<String, serde_yaml::Value>,
}

#[derive(Debug, Deserialize)]
struct NamedClient {
    #[serde(flatten)]
    name: HashMap<String, Client>,
}

#[derive(Debug, Deserialize)]
struct Config {
    clients: Vec<NamedClient>,
}

/// A tool to generate configuration URLs from YAML files
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the YAML configuration file
    yaml_file: String,

    /// Output file to write URLs to
    #[arg(short, long)]
    output: Option<String>,
}

fn replace_env_vars(value: &str) -> Result<String, String> {
    let re = Regex::new(r"\{\{([^}]+)\}\}").map_err(|e| e.to_string())?;
    let mut result = value.to_string();

    for cap in re.captures_iter(value) {
        let var_name = cap[1].trim();
        let env_value = env::var(var_name)
            .map_err(|_| format!("Environment variable {} not found", var_name))?;
        result = result.replace(&cap[0], &env_value);
    }

    Ok(result)
}

/// Recursively processes a YAML value, replacing environment variables in string values.
/// Environment variables should be in the format {{VAR_NAME}}.
/// For strings containing environment variables, replaces them with their values.
/// For mappings and sequences, recursively processes their elements.
/// For other value types (numbers, booleans, null), returns them unchanged.
fn process_value(value: &serde_yaml::Value) -> Result<serde_yaml::Value, String> {
    match value {
        serde_yaml::Value::String(s) => {
            if s.contains("{{") && s.contains("}}") {
                Ok(serde_yaml::Value::String(replace_env_vars(s)?))
            } else {
                Ok(value.clone())
            }
        }
        serde_yaml::Value::Mapping(map) => {
            let mut new_map = serde_yaml::Mapping::new();
            for (k, v) in map {
                new_map.insert(k.clone(), process_value(v)?);
            }
            Ok(serde_yaml::Value::Mapping(new_map))
        }
        serde_yaml::Value::Sequence(seq) => {
            let new_seq: Result<Vec<_>, _> = seq.iter().map(|v| process_value(v)).collect();
            Ok(serde_yaml::Value::Sequence(new_seq?))
        }
        _ => Ok(value.clone()),
    }
}

fn generate_url(client_name: &str, client: &Client) -> Result<String, String> {
    let protocol = client.protocol.to_lowercase();
    let mut url = Url::parse(&format!(
        "{}://{}@{}:{}",
        protocol,
        urlencoding::encode(&client.id),
        client.address,
        client.port
    ))
    .map_err(|e| format!("Invalid URL format: {}", e))?;

    // Process parameters
    let mut query_pairs = Vec::new();
    for (key, value) in &client.params {
        let encoded_value = match value {
            serde_yaml::Value::String(s) => urlencoding::encode(s).to_string(),
            serde_yaml::Value::Mapping(_) | serde_yaml::Value::Sequence(_) => {
                let json = serde_json::to_string(value).map_err(|e| e.to_string())?;
                urlencoding::encode(&json).to_string()
            }
            _ => serde_yaml::to_string(value).map_err(|e| e.to_string())?,
        };
        query_pairs.push(format!("{}={}", key, encoded_value));
    }

    // Add query string
    url.set_query(Some(&query_pairs.join("&")));

    // Add fragment (client name)
    url.set_fragment(Some(&urlencoding::encode(client_name)));

    Ok(url.to_string())
}

fn print_url_with_qr(name: &str, url: &str) {
    let qrstring = QrCode::new(url)
        .unwrap()
        .render::<Dense1x2>()
        .dark_color(unicode::Dense1x2::Light)
        .light_color(unicode::Dense1x2::Dark)
        .build();
    println!("{}:\n{}\n{}\n", name, url, qrstring);
}

fn write_urls_to_file(file: &mut fs::File, urls: &HashMap<String, String>) -> Result<(), String> {
    for url in urls.values() {
        writeln!(file, "{}", url).map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn main() -> Result<(), String> {
    let args = Args::parse();
    let mut urls = HashMap::new();

    let yaml_content = fs::read_to_string(&args.yaml_file).map_err(|e| e.to_string())?;
    let mut config: Config = serde_yaml::from_str(&yaml_content).map_err(|e| e.to_string())?;

    // Process environment variables in the config
    for named_client in &mut config.clients {
        for (name, client) in named_client.name.iter_mut() {
            client.id = replace_env_vars(&client.id)?;
            client.address = replace_env_vars(&client.address)?;

            let processed_params =
                process_value(&serde_yaml::to_value(&client.params).map_err(|e| e.to_string())?)?;
            client.params = serde_yaml::from_value(processed_params).map_err(|e| e.to_string())?;

            // Generate URL
            let url = generate_url(name, client)?;

            // Store URL with client name
            urls.insert(name.clone(), url);
        }
    }

    for (name, url) in &urls {
        print_url_with_qr(name, url);
    }

    // Write all URLs to file if specified
    if let Some(output_path) = args.output {
        let mut file = fs::File::create(output_path).map_err(|e| e.to_string())?;
        write_urls_to_file(&mut file, &urls)?;
    }

    Ok(())
}
