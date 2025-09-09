/*
 * SPDX-FileCopyrightText: 2025 2025 Chen Linxuan <me@black-desk.cn>
 *
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use clap::{Parser, Subcommand};

mod commands;
mod utils;

#[derive(Parser)]
#[command(name = "git-bp")]
#[command(about = "A Git tool for backporting commits between branches")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Sort commits in topological order
    Sort(commands::sort::Args),
    /// Generate git cherry-pick commands
    Pick(commands::pick::Args),
    /// Install vim syntax support files
    Vim(commands::vim::Args),
    /// Find fixes for commits on a reference branch
    Fix(commands::fix::Args),
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Sort(args) => {
            commands::sort::command(args)?;
        }
        Commands::Pick(args) => {
            commands::pick::command(args)?;
        }
        Commands::Vim(args) => {
            commands::vim::command(args)?;
        }
        Commands::Fix(args) => {
            commands::fix::command(args)?;
        }
    }

    Ok(())
}
