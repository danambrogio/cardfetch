#[macro_use]
extern crate clap;
extern crate surf;
extern crate serde_json;

use std::fs::File;
use std::io::Write;
use clap::App;
use async_std::task;
use serde_json::Value;

fn main() -> Result<(), surf::Exception> {
  let yaml = load_yaml!("../cli.yml");
  let matches = App::from_yaml(yaml).get_matches();

  let card_name = matches.value_of("CARD").unwrap();

  // Download the card image
  task::block_on(get_card(card_name))
}

async fn get_card(name: &str) -> Result<(), surf::Exception> {
  let url = format!("https://api.magicthegathering.io/v1/cards?name=\"{}\"", name);
  let body: String = surf::get(url).recv_string().await?;
  let json = parse_json(&body);

  let card_url = match json.get("cards").and_then(|value| value.get(0)).and_then(|value| value.get("imageUrl")).and_then(|value| value.as_str()) {
    Some(x) => x,
    None => "",
  };
  let mut https_url = String::from(card_url);
  https_url.insert(4, 's'); //convert to https

  let image = surf::get(https_url).await?.body_bytes().await?;
  let mut out: std::fs::File = File::create("card.png").expect("failed to create file");
  out.write_all(&image)?;
  
  Ok(())
}

fn parse_json(json: &str) -> serde_json::Value {
  let root: Value = match serde_json::from_str(json) {
    Ok(val) => val,
    Err(_) => serde_json::Value::Null,
  };

  root
}