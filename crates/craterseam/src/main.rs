use clap::{Parser, Subcommand};
use craterseam::{
    audit, baseline,
    io::{self, Result},
    model::*,
    report, server,
};
use std::path::PathBuf;
#[derive(Parser)]
#[command(
    name = "craterseam",
    version,
    about = "Planetary coordinate data debugger"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}
#[derive(Subcommand)]
enum Command {
    Fetch {
        #[arg(long,default_value="robbins",value_parser=["robbins"])]
        source: String,
        #[arg(long, default_value = "data/lunar_craters.parquet")]
        output: PathBuf,
        #[arg(long)]
        revision: Option<String>,
    },
    Audit {
        #[arg(long)]
        input: PathBuf,
        #[arg(long)]
        profile: Option<PathBuf>,
        #[arg(long, default_value = "artifacts")]
        output: PathBuf,
    },
    Serve {
        #[arg(long)]
        input: PathBuf,
        #[arg(long)]
        profile: Option<PathBuf>,
        #[arg(long, default_value_t = 8787)]
        port: u16,
        #[arg(long, default_value = "web/dist")]
        web: PathBuf,
    },
    Check {
        #[arg(long)]
        input: PathBuf,
        #[arg(long)]
        profile: Option<PathBuf>,
        #[arg(long)]
        baseline: PathBuf,
        #[arg(long, default_value_t = 0)]
        max_new_critical: usize,
    },
}
fn profile(path: Option<PathBuf>) -> Result<CoordinateProfile> {
    match path {
        Some(p) => Ok(serde_json::from_slice(&std::fs::read(p)?)?),
        None => Ok(CoordinateProfile::default()),
    }
}
async fn run() -> Result<bool> {
    match Cli::parse().command {
        Command::Fetch {
            output, revision, ..
        } => {
            io::fetch(&output, revision.as_deref()).await?;
            println!("Fetched {}", output.display());
        }
        Command::Audit {
            input,
            profile: p,
            output,
        } => {
            let p = profile(p)?;
            let a = audit::audit(&io::load(&input, &p)?, &p);
            report::write(&a, &output)?;
            println!("{}", serde_json::to_string_pretty(&a.summary)?);
        }
        Command::Serve {
            input,
            profile: p,
            port,
            web,
        } => {
            let p = profile(p)?;
            server::serve(io::load(&input, &p)?, p, port, web).await?;
        }
        Command::Check {
            input,
            profile: p,
            baseline: b,
            max_new_critical,
        } => {
            let p = profile(p)?;
            let a = audit::audit(&io::load(&input, &p)?, &p);
            let b = serde_json::from_slice(&std::fs::read(b)?)?;
            let comparison = baseline::compare(&a, &b, max_new_critical);
            println!("{}", serde_json::to_string_pretty(&comparison)?);
            return Ok(comparison.passed);
        }
    }
    Ok(true)
}
#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .with_writer(std::io::stderr)
        .init();
    match run().await {
        Ok(true) => {}
        Ok(false) => std::process::exit(2),
        Err(e) => {
            eprintln!("craterseam: {e}");
            std::process::exit(1);
        }
    }
}
