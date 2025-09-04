/*
 * SPDX-FileCopyrightText: 2025 2025 Chen Linxuan <me@black-desk.cn>
 *
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use std::collections::HashMap;
use std::collections::HashSet;
use std::process::Command;
use crate::utils::commits::CommitsParser;

#[derive(clap::Args)]
pub struct Args {
    /// Commit hashes to sort
    #[arg(required_unless_present = "commits_file")]
    pub commits: Vec<String>,

    /// File containing commit hashes to sort (one per line)
    #[arg(long = "commits-file", short = 'F', conflicts_with = "commits")]
    pub commits_file: Option<String>,

    /// Write sorted commits back to the input file (only works with --commits-file)
    #[arg(long = "in-place", short = 'i', requires = "commits_file")]
    pub in_place: bool,

    /// Reference point to sort commits
    #[arg(long = "ref", default_value = "HEAD")]
    pub reference: String,
}

/// Handle the sort command - sort commits in topological order
pub fn command(args: Args) -> Result<(), Box<dyn std::error::Error>> {
    // Get commit list from either command line args or file
    let (commit_infos, file_path) = CommitsParser::get_commits(args.commits, args.commits_file)?;

    // Extract hashes for sorting
    let commit_hashes = CommitsParser::extract_hashes(&commit_infos);
    let sorted_hashes = sort_commits_topologically(commit_hashes, &args.reference)?;

    // Create sorted CommitInfo vector, preserving original Change-Id and title information
    let mut sorted_commits_info = Vec::new();
    for sorted_hash in &sorted_hashes {
        if let Some(original_info) = commit_infos.iter().find(|info|
            info.hash.starts_with(sorted_hash) || sorted_hash.starts_with(&info.hash)) {
            let mut new_info = original_info.clone();
            new_info.hash = sorted_hash.clone(); // Use the hash format from user input
            sorted_commits_info.push(new_info);
        }
    }

    // Output results: either write back to file (in-place mode) or output to stdout
    match file_path {
        Some(file_path) => {
            if args.in_place {
                // Read entries with comments
                let (modelines, entries) = CommitsParser::read_from_file(&file_path)?;

                // Create a hash map for sorted commits order using flexible matching
                let sorted_map: HashMap<String, usize> = HashMap::new();
                let mut sorted_map = sorted_map;
                for (i, sorted_commit) in sorted_commits_info.iter().enumerate() {
                    // Map both the full hash and any original hash that matches
                    sorted_map.insert(sorted_commit.hash.clone(), i);
                    for entry in &entries {
                        if entry.commit.hash.starts_with(&sorted_commit.hash) ||
                           sorted_commit.hash.starts_with(&entry.commit.hash) {
                            sorted_map.insert(entry.commit.hash.clone(), i);
                        }
                    }
                }

                // Sort entries by the order in sorted_commits_info, keeping comments with commits
                let mut sorted_entries = entries.clone();
                sorted_entries.sort_by(|a, b| {
                    let a_idx = sorted_map.get(&a.commit.hash).unwrap_or(&999999);
                    let b_idx = sorted_map.get(&b.commit.hash).unwrap_or(&999999);
                    a_idx.cmp(b_idx)
                });

                // Update entries with enriched commit information
                for entry in &mut sorted_entries {
                    if let Some(sorted_commit) = sorted_commits_info.iter().find(|c|
                        c.hash.starts_with(&entry.commit.hash) ||
                        entry.commit.hash.starts_with(&c.hash)) {
                        entry.commit = sorted_commit.clone();
                    }
                }

                CommitsParser::write_to_file(&file_path, &modelines, &sorted_entries)?;
                println!("Updated {} commits in {}", sorted_commits_info.len(), file_path);
            } else {
                // Print to stdout
                for commit in &sorted_commits_info {
                    println!("{}", commit.to_line());
                }
            }
        }
        None => {
            // CLI commits - print to stdout
            for commit in &sorted_commits_info {
                println!("{}", commit.to_line());
            }
        }
    }

    Ok(())
}

fn sort_commits_topologically(
    input_commits: Vec<String>,
    reference: &str,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let output = Command::new("git")
        .args(["rev-list", "--topo-order", reference])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("git rev-list: {}", stderr).into());
    }

    let all_commits = String::from_utf8_lossy(&output.stdout);
    let commit_list: Vec<&str> = all_commits.lines().collect();

    let mut input_set: HashSet<String> = input_commits.into_iter().collect();

    let mut sorted_commits = Vec::new();
    for commit in commit_list {
        let commit = commit.trim();
        if commit.is_empty() {
            continue;
        }

        if input_set.is_empty() {
            continue;
        }

        let mut matched_input = None;
        for input_commit in input_set.iter() {
            if commit.starts_with(input_commit.as_str()) || input_commit.starts_with(commit) {
                matched_input = Some(input_commit.clone());
                break;
            }
        }

        if let Some(matched) = matched_input {
            sorted_commits.push(matched.clone());
            input_set.remove(&matched); // 移除已找到的提交，避免重复
        }
    }

    // 检查是否所有输入的提交都被找到
    if !input_set.is_empty() {
        let missing: Vec<_> = input_set.iter().collect();
        return Err(format!("Some commits not found in HEAD history: {:?}", missing).into());
    }

    Ok(sorted_commits)
}
