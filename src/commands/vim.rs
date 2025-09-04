/*
 * SPDX-FileCopyrightText: 2025 Chen Linxuan <me@black-desk.cn>
 *
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use std::fs;
use std::path::Path;

// Embed the vim plugin file content at compile time
const VIM_PLUGIN_CONTENT: &str = include_str!("../../vim/ftplugin/gitbackportcommits.vim");

#[derive(clap::Args)]
pub struct Args {
    /// Target vim configuration directory (default: auto-detect ~/.vim or ~/.config/nvim)
    #[arg(long = "vim-dir")]
    pub vim_dir: Option<String>,

    /// Force overwrite if file already exists
    #[arg(long = "force", short = 'f')]
    pub force: bool,
}

/// Handle the vim command - install vim syntax support files
pub fn command(args: Args) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(dir) = args.vim_dir {
        // Install to user-specified directory
        return install_to_vim_dir(&dir, args.force);
    }

    // Install to default vim and neovim directories
    let home = std::env::var("HOME")?;

    // Try neovim first (respect XDG_CONFIG_HOME)
    let nvim_dir = get_neovim_config_dir()?;
    // Always try to install to neovim directory (create if needed)
    install_to_vim_dir(&nvim_dir.to_string_lossy(), args.force)?;

    // Always try vim (it will create directory if needed)
    let vim_dir = Path::new(&home).join(".vim");
    install_to_vim_dir(&vim_dir.to_string_lossy(), args.force)
}

/// Get neovim configuration directory, respecting XDG_CONFIG_HOME
fn get_neovim_config_dir() -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    if let Ok(xdg_config_home) = std::env::var("XDG_CONFIG_HOME") {
        // Use XDG_CONFIG_HOME if set
        Ok(Path::new(&xdg_config_home).join("nvim"))
    } else {
        // Fall back to default ~/.config/nvim
        let home = std::env::var("HOME")?;
        Ok(Path::new(&home).join(".config").join("nvim"))
    }
}

/// Install vim plugin to a specific vim configuration directory
fn install_to_vim_dir(vim_config_dir: &str, force: bool) -> Result<(), Box<dyn std::error::Error>> {
    let target_file = Path::new(vim_config_dir).join("ftplugin").join("gitbackportcommits.vim");

    // Check if file exists and --force is not used
    if target_file.exists() && !force {
        return Ok(());
    }

    // Ensure target directory exists
    if let Some(parent) = target_file.parent() {
        fs::create_dir_all(parent)?;
    }

    // Write the embedded content to the target file
    fs::write(&target_file, VIM_PLUGIN_CONTENT)?;

    Ok(())
}
