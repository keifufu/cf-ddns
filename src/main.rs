use config::Config;
use serde_json::Value;

fn main() {
  let config = Config::builder()
    .add_source(config::File::with_name("config.toml"))
    .build()
    .unwrap();

  let api_key = config.get_string("API_KEY").unwrap();
  let zone_id = config.get_string("ZONE_ID").unwrap();
  let records = config
    .get_array("RECORDS")
    .unwrap()
    .iter()
    .map(|value| value.to_string())
    .collect::<Vec<String>>();

  loop {
    update_cloudflare(&zone_id, &api_key, &records);
    std::thread::sleep(std::time::Duration::from_secs(5 * 60));
  }
}

fn update_cloudflare(zone_id: &String, api_key: &String, records_to_update: &[String]) {
  let current_ip = get_current_ip();
  let valid = current_ip
      .as_ref()
      .map(|ip| {
          regex::Regex::new(r"(\b25[0-5]|\b2[0-4][0-9]|\b[01]?[0-9][0-9]?)(\.(25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)){3}")
              .unwrap()
              .is_match(ip)
      })
      .unwrap_or(false);

  if !valid {
    return;
  }
  let ip = current_ip.unwrap();

  let records = get_dns_records(zone_id, api_key);

  if let Some(records) = records {
    for result in records {
      if result["type"] != "A" {
        continue;
      }
      let name = result["name"].as_str().to_owned().unwrap().to_string();
      if !records_to_update.contains(&name.to_string()) {
        continue;
      }
      println!("Updating: '{}'", name);
      if update_record(
        zone_id,
        &result["id"].as_str().unwrap().to_string(),
        &ip,
        api_key,
      )
      .is_err()
      {
        println!("Failed to update {}", name);
      }
    }
  }

  println!("Updated IP to {}", ip);
}

fn get_current_ip() -> Option<String> {
  let response = reqwest::blocking::get("https://api.ipify.org")
    .ok()?
    .text()
    .ok()?;

  Some(response)
}

fn get_dns_records(zone_id: &String, api_key: &String) -> Option<Vec<Value>> {
  let client = reqwest::blocking::Client::new();
  let response = client
    .get(format!(
      "https://api.cloudflare.com/client/v4/zones/{}/dns_records",
      zone_id
    ))
    .header("Authorization", format!("Bearer {}", api_key))
    .header("Accept", "application/json")
    .send()
    .ok()?;

  let json: Value = response.json().ok()?;
  let records = json["result"].as_array()?.clone();

  Some(records)
}

fn update_record(
  zone_id: &String,
  id: &String,
  ip: &String,
  api_key: &String,
) -> Result<(), Box<dyn std::error::Error>> {
  let client = reqwest::blocking::Client::new();
  client
    .patch(format!(
      "https://api.cloudflare.com/client/v4/zones/{}/dns_records/{}",
      zone_id, id
    ))
    .header("Authorization", format!("Bearer {}", api_key))
    .header("Content-Type", "application/json")
    .json(&serde_json::json!({
      "content": ip
    }))
    .send()?;

  Ok(())
}
