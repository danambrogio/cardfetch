#[macro_use]
extern crate clap;
extern crate surf;
extern crate serde_json;
extern crate image;
extern crate url;

use async_std::task;
use clap::App;
use serde_json::Value;
use std::io::Cursor;
use std::str::from_utf8;
use url::form_urlencoded::{byte_serialize};

fn main() -> Result<(), surf::Exception> {
  let yaml = load_yaml!("../cli.yml");
  let matches = App::from_yaml(yaml).get_matches();

  let card_name = matches.value_of("CARD").unwrap();
  task::block_on(get_card(card_name)).unwrap();

  Ok(())
}

async fn get_card(name: &str) -> Result<(), surf::Exception> {
  let cardname: String = byte_serialize(name.as_bytes()).collect();
  let url = format!("https://api.magicthegathering.io/v1/cards?name=\"{}\"", cardname);
  let body: String = surf::get(url).recv_string().await?;
  let json = parse_json(&body);

  
  let mut card_url = get_card_url(json);
  card_url.insert(4, 's'); //convert to https

  let image = surf::get(card_url).await?.body_bytes().await?;
  print_card(image);

  Ok(())
}

fn parse_json(json: &str) -> serde_json::Value {
  let root: Value = match serde_json::from_str(json) {
    Ok(val) => val,
    Err(_) => serde_json::Value::Null,
  };

  root
}

// Gets the `imageUrl` of the first card that has one
fn get_card_url(json: Value) -> String {
  let cards = json["cards"].as_array().unwrap();
  let c: Vec<Value> = cards.clone();
  let cards_with_images: Vec<Value> = c.into_iter().filter(|card| card["imageUrl"].is_string()).collect();
  // using indexes here can fail
  let x = match cards_with_images[0]["imageUrl"].as_str() {
    Some(y) => y,
    None => "",
  };
  String::from(x)
}

// Shamelessly stolen from
// https://github.com/edelsonc/asciify/blob/master/src/main.rs
fn print_card(img: Vec<u8>) -> () {
  let image = image::io::Reader::new(Cursor::new(img))
    .with_guessed_format().unwrap().decode().unwrap();

  let img = image.resize_exact(94, 47, image::FilterType::Nearest);

  let imgbuf = img.to_luma();
  let ascii_art = imgbuf.pixels()
                  .map(|p| intensity_to_ascii(&p[0]) )
                  .fold( String::new(), |s, p| s + p );

  // we have one long string, but we need to chunk it by line
  let subs = ascii_art.as_bytes()
      .chunks(imgbuf.width() as usize)
      .map(from_utf8)
      .collect::<Result<Vec<&str>, _>>()
      .unwrap();
  for s in subs {
      println!("{}", s);
  }
}

fn intensity_to_ascii(value: &u8) -> &str {
  // changes an intensity into an ascii character
  let ascii_chars  = [
      " ", ".", "^", ",", ":", "_", "=", "~", "+", "O", "o", "*",
      "#", "&", "%", "B", "@", "$"
  ];
  
  let n_chars = ascii_chars.len() as u8;
  let step = 255u8 / n_chars;
  for i in 1..(n_chars - 1) {   
      let comp = &step * i;
      if value < &comp {
          let idx = (i - 1) as usize;
          return ascii_chars[idx]
      }
  }

  ascii_chars[ (n_chars - 1) as usize ]
}