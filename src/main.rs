use clap::Command;
use local_ip_address::local_ip;
use log::{debug, error};
use serde_json::Value;
use std::fs;
use std::path::Path;
use std::time::SystemTime;
use ursual::{configure_logging, debug_arg, verbose_arg};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  let args = Command::new("powerbars")
    .about("Rust based status bar helper")
    .subcommand_required(true)
    .arg(debug_arg())
    .arg(verbose_arg())
    .subcommand(Command::new("ip").about("Get local IP address"))
    .subcommand(Command::new("extip").about("Get external IP address"))
    .subcommand(Command::new("temperature").about("Get current outside temperature"))
    .get_matches();

  configure_logging(&args);

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
      let path = Path::new("weather.json");
      let metadata = fs::metadata(path)?;

      let mut refresh = true;
      if let Ok(modified) = metadata.modified() {
        let now = SystemTime::now();
        let diff = now.duration_since(modified).unwrap();
        if diff < std::time::Duration::from_secs(60 * 5) {
          refresh = false
        }
      }

      let text = if refresh {
        let env_name = "WEATHER_API_KEY";
        let location = "Amsterdam";
        let api_key = std::env::var(env_name).unwrap_or_else(|_| panic!("Missing {} environment variable", env_name));
        let url = format!("http://api.weatherapi.com/v1/current.json?key={}&q={}&aqi=no", api_key, location);
        let text = reqwest::get(url).await?.text().await?;
        fs::write(path, &text)?;
        debug!("Wrote to {:?}", path);
        text
      } else {
        fs::read_to_string(path)?
      };

      let resp: Value = serde_json::from_str(&text)?;
      debug!("{:?}", resp);
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
