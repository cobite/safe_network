// Copyright 2024 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

mod terminal;

use clap::Parser;
use color_eyre::eyre::Result;
use node_launchpad::{
    app::App,
    config::configure_winsw,
    utils::{initialize_logging, initialize_panic_handler, version},
};
use sn_peers_acquisition::PeersArgs;
use std::{env, path::PathBuf};
use tokio::task::LocalSet;

#[derive(Parser, Debug)]
#[command(author, version = version(), about)]
pub struct Cli {
    #[arg(
        short,
        long,
        value_name = "FLOAT",
        help = "Tick rate, i.e. number of ticks per second",
        default_value_t = 1.0
    )]
    pub tick_rate: f64,

    #[arg(
        short,
        long,
        value_name = "FLOAT",
        help = "Frame rate, i.e. number of frames per second",
        default_value_t = 60.0
    )]
    pub frame_rate: f64,

    /// Provide a path for the safenode binary to be used by the service.
    ///
    /// Useful for creating the service using a custom built binary.
    #[clap(long)]
    safenode_path: Option<PathBuf>,

    #[command(flatten)]
    pub(crate) peers: PeersArgs,
}

async fn tokio_main() -> Result<()> {
    initialize_logging()?;

    initialize_panic_handler()?;

    let args = Cli::parse();

    let mut app = App::new(
        args.tick_rate,
        args.frame_rate,
        args.peers,
        args.safenode_path,
    )?;
    app.run().await?;

    Ok(())
}

fn is_running_in_terminal() -> bool {
    atty::is(atty::Stream::Stdout)
}

#[tokio::main]
async fn main() -> Result<()> {
    configure_winsw().await?;

    if !is_running_in_terminal() {
        // If we weren't already running in a terminal, this process returns early, having spawned
        // a new process that launches a terminal.
        let terminal_type = terminal::detect_and_setup_terminal()?;
        terminal::launch_terminal(&terminal_type)?;
        return Ok(());
    }

    // Construct a local task set that can run `!Send` futures.
    let local = LocalSet::new();
    local
        .run_until(async {
            if let Err(e) = tokio_main().await {
                eprintln!("{} failed:", env!("CARGO_PKG_NAME"));

                Err(e)
            } else {
                Ok(())
            }
        })
        .await
}
