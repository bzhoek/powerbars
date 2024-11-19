use clap::Command;
use local_ip_address::local_ip;
use log::error;
use serde_json::Value;
use ursual::{debug_arg, verbose_arg};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  let args = Command::new("id3-rs")
    .about("Rust based ID3 tagging")
    .subcommand_required(true)
    .arg(debug_arg())
    .arg(verbose_arg())
    .subcommand(Command::new("ip").about("Get IP address"))
    .subcommand(Command::new("extip").about("Get external IP address"))
    .subcommand(Command::new("temperature").about("Get current outside temperature"))
    .get_matches();

  match args.subcommand() {
    Some(("extip", _)) => {
      let resp = reqwest::get("https://httpbin.org/ip").await?.json::<Value>().await?;
      println!("{}", resp["origin"].as_str().unwrap());
    }
    Some(("ip", _)) => {
      let ip_addr = local_ip().unwrap();
      println!("{}", ip_addr);
    }
    Some(("temperature", _)) => {
      let env_name = "WEATHER_API_KEY";
      let location = "Amsterdam";
      let api_key = std::env::var(env_name).unwrap_or_else(|_| panic!("Missing {} environment variable", env_name));
      let url = format!("http://api.weatherapi.com/v1/current.json?key={}&q={}&aqi=no", api_key, location);
      let resp = reqwest::get(url).await?.json::<Value>().await?;
      let celcius = nested_value(&resp, vec!["current", "temp_c"]);
      println!("{}º", celcius);
    }
    cmd => error!("Unknown command {:?}", cmd),
  }

  Ok(())
}

fn nested_value<'a>(value: &'a Value, keys: Vec<&'a str>) -> &'a Value {
  let mut current = value;
  for key in keys {
    current = &current[key];
  }
  current
}
