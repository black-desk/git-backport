/*
 * SPDX-FileCopyrightText: 2025 2025 Chen Linxuan <me@black-desk.cn>
 *
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use crate::utils::commits::CommitsParser;

#[derive(clap::Args)]
pub struct Args {
    /// Commit hashes to generate cherry-pick commands for
    #[arg(required_unless_present = "commits_file")]
    pub commits: Vec<String>,

    /// File containing commit hashes to cherry-pick (one per line)
    #[arg(long = "commits-file", short = 'F', conflicts_with = "commits")]
    pub commits_file: Option<String>,
}

/// Handle the pick command - generate git cherry-pick commands
pub fn command(args: Args) -> Result<(), Box<dyn std::error::Error>> {
    // 获取commit列表：要么从命令行参数，要么从文件
    let (commit_infos, _) = CommitsParser::get_commits(args.commits, args.commits_file)?;

    // 生成cherry-pick命令
    for commit_info in commit_infos {
        println!("git cherry-pick -x --signoff {}", commit_info.hash);
    }

    Ok(())
}
