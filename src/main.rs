#[macro_use]
extern crate clap;
extern crate surf;
extern crate serde_json;
extern crate image;

use std::str::from_utf8;
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
  task::block_on(get_card(card_name)).unwrap();
  print_card();

  Ok(())
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
  let mut out: std::fs::File = File::create("card.jpeg").expect("failed to create file");
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

fn print_card() -> () {
  let img = match image::open("./card.jpeg") {
    Ok(p) => p,
    Err(e) => panic!("Not a valid image path or could not open image. {}", e),
  };

  let img = img.resize_exact(120, 60, image::FilterType::Nearest);

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
  // this is a central step in creating the ascii art
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