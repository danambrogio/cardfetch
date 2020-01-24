#[macro_use]
extern crate clap;
extern crate surf;

use clap::App;
use async_std::task;

fn main() -> Result<(), surf::Exception> {
  let yaml = load_yaml!("../cli.yml");
  let matches = App::from_yaml(yaml).get_matches();

  let card_name = matches.value_of("CARD").unwrap();

  task::block_on(get_card(card_name))
}

async fn get_card(name: &str) -> Result<(), surf::Exception> {
  let url = format!("https://api.magicthegathering.io/v1/cards?name={}", name);
  let body: String = surf::get(url).recv_string().await?;

  println!("{}", body);

  Ok(())
}