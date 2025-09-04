/*
 * SPDX-FileCopyrightText: 2025 2025 Chen Linxuan <me@black-desk.cn>
 *
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use std::fs;
use std::process::Command;

/// Represents a commit entry with optional preceding comments
#[derive(Clone, Debug)]
pub struct CommitEntry {
    /// Comments that precede this commit (including the commit they belong to)
    pub comments: Vec<String>,
    /// The commit information
    pub commit: CommitInfo,
}

impl CommitEntry {
    pub fn with_comments(commit: CommitInfo, comments: Vec<String>) -> Self {
        Self {
            comments,
            commit,
        }
    }

    /// Format the entry back to file lines
    pub fn to_lines(&self) -> Vec<String> {
        let mut lines = Vec::new();
        lines.extend(self.comments.clone());
        lines.push(self.commit.to_line());
        lines
    }
}

/// Represents a commit with its hash, optional Change-Id, and optional title
#[derive(Clone, Debug)]
pub struct CommitInfo {
    pub hash: String,
    pub change_id: Option<String>,
    pub title: Option<String>,
}

impl CommitInfo {
    /// Create a new CommitInfo from just a hash
    pub fn from_hash(hash: String) -> Self {
        Self {
            hash,
            change_id: None,
            title: None,
        }
    }

    /// Parse a line from the commit file format: "hash [Change-Id] [title]"
    pub fn parse_line(line: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let line = line.trim();
        if line.is_empty() {
            return Err("Empty line".into());
        }

        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() {
            return Err("No commit hash found".into());
        }

        let hash = parts[0].to_string();
        let mut change_id = None;
        let mut title_parts = Vec::new();

        // Parse remaining parts
        for part in &parts[1..] {
            if part.starts_with("I") && part.len() == 41 {
                // Looks like a Gerrit Change-Id
                change_id = Some(part.to_string());
            } else {
                title_parts.push(*part);
            }
        }

        let title = if title_parts.is_empty() {
            None
        } else {
            Some(title_parts.join(" "))
        };

        Ok(Self {
            hash,
            change_id,
            title,
        })
    }

    /// Format the commit info back to file line format
    pub fn to_line(&self) -> String {
        let mut parts = vec![self.hash.clone()];

        if let Some(change_id) = &self.change_id {
            parts.push(change_id.clone());
        }

        if let Some(title) = &self.title {
            parts.push(title.clone());
        }

        parts.join(" ")
    }

    /// Get commit message title from git if not already set, and expand hash to full if needed
    pub fn fetch_title_if_missing(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // First, get the full commit hash if we have a short one
        self.expand_hash_to_full()?;

        if self.title.is_some() {
            return Ok(());
        }

        let output = Command::new("git")
            .args(["log", "--format=%s", "-n", "1", &self.hash])
            .output()?;

        if output.status.success() {
            let title = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !title.is_empty() {
                self.title = Some(title);
            }
        }

        Ok(())
    }

    /// Expand short commit hash to full hash
    pub fn expand_hash_to_full(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // If hash is already 40 characters (full SHA-1), no need to expand
        if self.hash.len() == 40 {
            return Ok(());
        }

        let output = Command::new("git")
            .args(["rev-parse", &self.hash])
            .output()?;

        if output.status.success() {
            let full_hash = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if full_hash.len() == 40 {
                self.hash = full_hash;
            }
        }

        Ok(())
    }

    /// Extract Change-Id from commit message if not already set
    pub fn fetch_change_id_if_missing(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.change_id.is_some() {
            return Ok(());
        }

        let output = Command::new("git")
            .args(["log", "--format=%B", "-n", "1", &self.hash])
            .output()?;

        if output.status.success() {
            let body = String::from_utf8_lossy(&output.stdout);
            for line in body.lines() {
                if line.starts_with("Change-Id: I") {
                    if let Some(change_id) = line.strip_prefix("Change-Id: ") {
                        self.change_id = Some(change_id.trim().to_string());
                        break;
                    }
                }
            }
        }

        Ok(())
    }
}

/// Utility for handling commit lists from files or command line arguments
pub struct CommitsParser;

impl CommitsParser {
    /// Read commit entries from a file, preserving comments
    pub fn read_from_file(file_path: &str) -> Result<(Vec<String>, Vec<CommitEntry>), Box<dyn std::error::Error>> {
        let content = fs::read_to_string(file_path)?;
        let lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();

        let mut modelines = Vec::new();
        let mut entries = Vec::new();
        let mut current_comments = Vec::new();
        let mut line_idx = 0;

        // Extract modelines from the beginning
        while line_idx < lines.len() {
            let line = lines[line_idx].trim();
            if line.starts_with("# vim:") || line.starts_with("# vi:") {
                modelines.push(lines[line_idx].clone());
                line_idx += 1;
            } else {
                break;
            }
        }

        // Process remaining lines
        while line_idx < lines.len() {
            let line = lines[line_idx].trim();

            if line.is_empty() {
                // Empty line - add to current comments if we have any
                if !current_comments.is_empty() || line_idx + 1 < lines.len() {
                    current_comments.push(lines[line_idx].clone());
                }
            } else if line.starts_with('#') {
                // Comment line
                current_comments.push(lines[line_idx].clone());
            } else {
                // This should be a commit line
                match CommitInfo::parse_line(line) {
                    Ok(commit) => {
                        entries.push(CommitEntry::with_comments(commit, current_comments.clone()));
                        current_comments.clear();
                    }
                    Err(_) => {
                        // If it's not a valid commit, treat it as a comment
                        current_comments.push(lines[line_idx].clone());
                    }
                }
            }
            line_idx += 1;
        }

        if entries.is_empty() {
            return Err("No commits found in file".into());
        }

        Ok((modelines, entries))
    }

    /// Write commit entries to a file, preserving comments and adding vim modeline
    pub fn write_to_file(file_path: &str, modelines: &[String], entries: &[CommitEntry]) -> Result<(), Box<dyn std::error::Error>> {
        let mut all_lines = Vec::new();

        // Add vim modeline if not already present
        if modelines.is_empty() {
            all_lines.push("# vim: ft=gitbackportcommits".to_string());
        } else {
            all_lines.extend(modelines.iter().cloned());
        }

        // Add entries with enriched information
        for entry in entries {
            let mut enriched_entry = entry.clone();
            enriched_entry.commit.fetch_change_id_if_missing()?;
            enriched_entry.commit.fetch_title_if_missing()?;

            let entry_lines = enriched_entry.to_lines();
            all_lines.extend(entry_lines);
        }

        let content = all_lines.join("\n") + "\n";
        fs::write(file_path, content)?;
        Ok(())
    }

    /// Get commits from either command line arguments or file
    pub fn get_commits(
        cli_commits: Vec<String>,
        commits_file: Option<String>
    ) -> Result<(Vec<CommitInfo>, Option<String>), Box<dyn std::error::Error>> {
        if let Some(file_path) = commits_file {
            let (_, entries) = Self::read_from_file(&file_path)?;
            let commits = entries.into_iter().map(|e| e.commit).collect();
            Ok((commits, Some(file_path)))
        } else {
            // Convert simple strings to CommitInfo
            let commits = cli_commits.into_iter()
                .map(CommitInfo::from_hash)
                .collect();
            Ok((commits, None))
        }
    }

    /// Extract just the hashes from CommitInfo vector (for backward compatibility)
    pub fn extract_hashes(commits: &[CommitInfo]) -> Vec<String> {
        commits.iter().map(|c| c.hash.clone()).collect()
    }
}
