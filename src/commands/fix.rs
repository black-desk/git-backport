/*
 * SPDX-FileCopyrightText: 2025 2025 Chen Linxuan <me@black-desk.cn>
 *
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use std::process::Command;
use log::{debug, warn};
use crate::utils::commits::CommitInfo;

#[derive(clap::Args)]
pub struct Args {
    /// Base commit to start checking from (exclusive)
    #[arg(long = "base", required = true)]
    pub base: String,

    /// Reference branch to search for fixes
    #[arg(long = "ref", required = true)]
    pub ref_branch: String,
}

/// Handle the fix command - find fixes for commits on a reference branch
pub fn command(args: Args) -> Result<(), Box<dyn std::error::Error>> {
    // Get commits in range base..HEAD
    let commits_in_range = get_commits_in_range(&args.base, "HEAD")?;

    if commits_in_range.is_empty() {
        debug!("No commits found in range {}..HEAD", args.base);
        return Ok(());
    }

    debug!("Found {} commits in range {}..HEAD", commits_in_range.len(), args.base);

    // Process each commit in the range
    let mut fix_commits = Vec::new();

    for mut commit in commits_in_range {
        // Enrich commit info
        commit.fetch_change_id_if_missing()?;
        commit.fetch_title_if_missing()?;

        debug!("Processing commit: {} {:?} {:?}",
               commit.hash, commit.change_id, commit.title);

        // Find original commit on ref branch
        if let Some(original_commit) = find_original_commit(&commit, &args.ref_branch)? {
            debug!("Found original commit for {}: {}", commit.hash, original_commit);

            // Search for fixes on ref branch
            let fixes = find_fixes_for_commit(&original_commit, &args.ref_branch)?;

            if !fixes.is_empty() {
                debug!("Found {} fix(es) for {}: {:?}", fixes.len(), original_commit, fixes);
                for fix_commit in fixes {
                    fix_commits.push(fix_commit);
                }
            }

            // Check for references that are not explicit fixes
            let references = find_references_for_commit(&original_commit, &args.ref_branch)?;
            debug!("Found {} references for {}: {:?}", references.len(), original_commit, references);
            for reference in references {
                debug!("Checking if reference {} is an explicit fix for {}", reference, original_commit);
                if !is_explicit_fix(&reference, &original_commit)? {
                    if let Some(ref_title) = get_commit_title(&reference)? {
                        warn!("Commit {} references {} but is not marked as a fix: {}",
                              reference, original_commit, ref_title);
                    } else {
                        warn!("Commit {} references {} but is not marked as a fix",
                              reference, original_commit);
                    }
                } else {
                    debug!("Reference {} is an explicit fix, skipping warning", reference);
                }
            }
        } else {
            debug!("Could not find original commit for {} on {}", commit.hash, args.ref_branch);
        }
    }

    // Generate commits file format and output to stdout
    output_commits_file(&fix_commits)?;

    Ok(())
}

/// Get commits in the specified range
fn get_commits_in_range(base: &str, head: &str) -> Result<Vec<CommitInfo>, Box<dyn std::error::Error>> {
    let range = format!("{}..{}", base, head);
    let output = Command::new("git")
        .args(["rev-list", "--reverse", &range])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("git rev-list failed: {}", stderr).into());
    }

    let commits_text = String::from_utf8_lossy(&output.stdout);
    let mut commits = Vec::new();

    for line in commits_text.lines() {
        let line = line.trim();
        if !line.is_empty() {
            commits.push(CommitInfo::from_hash(line.to_string()));
        }
    }

    Ok(commits)
}

/// Find the original commit on ref branch based on change-id or was-change-id
fn find_original_commit(commit: &CommitInfo, ref_branch: &str) -> Result<Option<String>, Box<dyn std::error::Error>> {
    if let Some(change_id) = &commit.change_id {
        // First try to find by change-id
        if let Some(original) = find_commit_by_change_id(change_id, ref_branch)? {
            return Ok(Some(original));
        }
    }

    // Try to find by was-change-id
    if let Some(was_change_id) = get_was_change_id(&commit.hash)? {
        if let Some(original) = find_commit_by_change_id(&was_change_id, ref_branch)? {
            return Ok(Some(original));
        }
    }

    Ok(None)
}

/// Find commit by change-id on specified branch
fn find_commit_by_change_id(change_id: &str, ref_branch: &str) -> Result<Option<String>, Box<dyn std::error::Error>> {
    let output = Command::new("git")
        .args(["log", "--format=%H", "--grep", &format!("Change-Id: {}", change_id), ref_branch])
        .output()?;

    if !output.status.success() {
        return Ok(None);
    }

    let commits_text = String::from_utf8_lossy(&output.stdout);
    for line in commits_text.lines() {
        let line = line.trim();
        if !line.is_empty() {
            return Ok(Some(line.to_string()));
        }
    }

    Ok(None)
}

/// Get was-change-id from commit message
fn get_was_change_id(commit_hash: &str) -> Result<Option<String>, Box<dyn std::error::Error>> {
    let output = Command::new("git")
        .args(["log", "--format=%B", "-n", "1", commit_hash])
        .output()?;

    if !output.status.success() {
        return Ok(None);
    }

    let body = String::from_utf8_lossy(&output.stdout);
    for line in body.lines() {
        if line.starts_with("Was-Change-Id: I") {
            if let Some(was_change_id) = line.strip_prefix("Was-Change-Id: ") {
                return Ok(Some(was_change_id.trim().to_string()));
            }
        }
    }

    Ok(None)
}

/// Find commits that fix the given commit
fn find_fixes_for_commit(original_commit: &str, ref_branch: &str) -> Result<Vec<CommitInfo>, Box<dyn std::error::Error>> {
    debug!("Searching for fixes for commit: {} on branch: {}", original_commit, ref_branch);

    // Search for commits that contain "Fixes: <commit_hash>" pattern
    // Only search commits that come after the original commit (since fixes can't appear before)
    let range = format!("{}..{}", original_commit, ref_branch);
    
    // Try both full hash and short hash patterns
    let short_hash = &original_commit[..std::cmp::min(12, original_commit.len())];
    
    debug!("Searching for fix patterns in range: {}", range);
    
    let output = Command::new("git")
        .args([
            "log", 
            "--format=%H", 
            "--grep", &format!("Fixes: {}", original_commit),
            "--grep", &format!("Fixes: {}", short_hash),
            &range
        ])
        .output()?;

    let mut fix_commits = Vec::new();

    if output.status.success() {
        let commits_text = String::from_utf8_lossy(&output.stdout);
        for line in commits_text.lines() {
            let line = line.trim();
            if !line.is_empty() {
                let mut commit_info = CommitInfo::from_hash(line.to_string());
                commit_info.fetch_change_id_if_missing()?;
                commit_info.fetch_title_if_missing()?;
                fix_commits.push(commit_info);
                debug!("Found fix commit: {} for {}", line, original_commit);
            }
        }
    }

    debug!("Found {} fix commits for {}", fix_commits.len(), original_commit);
    Ok(fix_commits)
}

/// Find commits that reference the given commit (but may not be explicit fixes)
fn find_references_for_commit(original_commit: &str, ref_branch: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    // Try both full hash and short hash (first 12 chars)
    let short_hash = &original_commit[..std::cmp::min(12, original_commit.len())];

    debug!("Searching for references using full hash: {} and short hash: {}", original_commit, short_hash);

    // Only search commits that come after the original commit (since references can't appear before)
    let range = format!("{}..{}", original_commit, ref_branch);
    let mut all_references = Vec::new();

    // Search with both full hash and short hash in one git command
    let mut args = vec![
        "log".to_string(),
        "--format=%H".to_string(),
        "--grep".to_string(),
        original_commit.to_string(),
    ];

    // Add short hash pattern if it's different from full hash
    if short_hash != original_commit {
        args.push("--grep".to_string());
        args.push(short_hash.to_string());
    }

    args.push(range);

    let output = Command::new("git")
        .args(&args)
        .output()?;

    if output.status.success() {
        let commits_text = String::from_utf8_lossy(&output.stdout);
        for line in commits_text.lines() {
            let line = line.trim();
            if !line.is_empty() {
                all_references.push(line.to_string());
            }
        }
    }

    Ok(all_references)
}

/// Check if a commit is an explicit fix for the original commit
fn is_explicit_fix(commit_hash: &str, original_commit: &str) -> Result<bool, Box<dyn std::error::Error>> {
    let output = Command::new("git")
        .args(["log", "--format=%B", "-n", "1", commit_hash])
        .output()?;

    if !output.status.success() {
        return Ok(false);
    }

    let body = String::from_utf8_lossy(&output.stdout);

    // Look for various fix patterns
    for line in body.lines() {
        let line = line.trim();

        debug!("Checking line: '{}' for fixes of {}", line, original_commit);

        // Pattern: "Fixes: <commit_hash>" (may be short hash)
        if line.starts_with("Fixes: ") {
            if let Some(fixes_part) = line.strip_prefix("Fixes: ") {
                // Extract the commit hash part (before any space or parenthesis)
                let fixes_hash = fixes_part.split_whitespace().next().unwrap_or("");

                // Check if the fix hash matches the original commit (either full or partial match)
                if original_commit.starts_with(fixes_hash) || fixes_hash.starts_with(original_commit) {
                    debug!("Found explicit fix pattern in line: {}", line);
                    return Ok(true);
                }
            }
        }
    }

    Ok(false)
}

/// Get commit title
fn get_commit_title(commit_hash: &str) -> Result<Option<String>, Box<dyn std::error::Error>> {
    let output = Command::new("git")
        .args(["log", "--format=%s", "-n", "1", commit_hash])
        .output()?;

    if !output.status.success() {
        return Ok(None);
    }

    let title = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if title.is_empty() {
        Ok(None)
    } else {
        Ok(Some(title))
    }
}

/// Output commits in file format to stdout
fn output_commits_file(commits: &[CommitInfo]) -> Result<(), Box<dyn std::error::Error>> {
    // Add vim modeline
    println!("# vim: ft=gitbackportcommits");

    // Output each commit
    for commit in commits {
        println!("{}", commit.to_line());
    }

    Ok(())
}
