use anyhow::Result;
use libwindi::client::ClientConfig;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "windi", about = "Bluebird WindiSync CLI client")]
struct Opt {
  /// Optional custom service URL.
  #[structopt(long = "service", short = "s")]
  service: Option<String>,

  /// Token.
  #[structopt(long, env = "WINDI_TOKEN")]
  token: String,

  #[structopt(subcommand)]
  cmd: Command,
}

#[derive(Debug, StructOpt)]
enum Command {
  /// Manually pull logs once from the provided sequence number.
  Pull {
    /// Hex-encoded start sequence number.
    #[structopt(long = "from", short = "f", default_value = "")]
    from: String,
  },
}

#[tokio::main]
async fn main() -> Result<()> {
  if std::env::var("RUST_LOG").is_err() {
    std::env::set_var("RUST_LOG", "info");
  }
  pretty_env_logger::init_timed();
  let opt = Opt::from_args();
  let mut config: ClientConfig = Default::default();
  config.url = opt.service;
  config.token = opt.token;
  let client = libwindi::client::Client::new(config)?;
  match opt.cmd {
    Command::Pull { from } => {
      let mut from = hex::decode(&from)?;
      from.reverse();
      from.resize(16, 0);
      let mut from = u128::from_le_bytes(<[u8; 16]>::try_from(from).unwrap());
      loop {
        let rsp = client.sync(from).await?;
        if rsp.is_empty() {
          break;
        }
        from = rsp.last().unwrap().seq + 1;

        for log in rsp {
          let value: serde_json::Value = match log.log {
            Ok(x) => serde_json::to_value(&x)?,
            Err(x) => serde_json::Value::String(x),
          };
          let entry = serde_json::json!({
            "seq": hex::encode(&log.seq.to_be_bytes()[..]),
            "value": value,
          });
          println!("{}", entry.to_string());
        }
      }
    }
  }
  Ok(())
}
