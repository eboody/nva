use app::{agents, tools};
use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "pet-resort")]
#[command(about = "Pet resort agent-foundation design CLI")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Print baseline agent specs as JSON.
    Agents,
    /// Print external tool candidates as JSON.
    Tools,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Agents => println!(
            "{}",
            serde_json::to_string_pretty(&agents::baseline_agent_specs())?
        ),
        Command::Tools => {
            let tools = vec![
                tools::ExternalToolCandidate::GingrPortal,
                tools::ExternalToolCandidate::PaymentProvider,
                tools::ExternalToolCandidate::SmsProvider,
                tools::ExternalToolCandidate::EmailProvider,
                tools::ExternalToolCandidate::FileStorage,
                tools::ExternalToolCandidate::OcrOrDocumentAi,
                tools::ExternalToolCandidate::CameraOrWebcamProvider,
                tools::ExternalToolCandidate::HermesKanban,
                tools::ExternalToolCandidate::HermesCronOrWebhook,
                tools::ExternalToolCandidate::Postgres,
            ];
            println!("{}", serde_json::to_string_pretty(&tools)?);
        }
    }
    Ok(())
}
