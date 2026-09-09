use clap::Parser;
use rsmcp::{cli, server};

#[tokio::main]
async fn main() {
    let mut arguments = cli::CommandArguments::parse();

    if let Err(err) = arguments.validate() {
        eprintln!("Error: {err}");
        return;
    };

    if let Err(error) = server::start_server(arguments).await {
        eprintln!("{error}");
    }
}
