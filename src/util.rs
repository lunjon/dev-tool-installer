use anyhow::{bail, Result};
use serde::de::DeserializeOwned;
use std::ffi::OsStr;
use std::io::{LineWriter, Read, Write};
use std::os::unix::prelude::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::{fs, process};

pub fn json_from_file<T>(path: &Path) -> Result<T>
where
    T: DeserializeOwned,
{
    let mut file = fs::OpenOptions::new().read(true).open(path)?;
    let t: T = serde_json::from_reader(&mut file)?;
    Ok(t)
}

pub fn toml_from_file<T>(path: &Path) -> Result<T>
where
    T: DeserializeOwned,
{
    let mut file = fs::OpenOptions::new().read(true).open(path)?;
    let mut buf = String::new();
    file.read_to_string(&mut buf)?;

    let t: T = toml::from_str(&buf)?;
    Ok(t)
}

/// Ensures that a command, e.g. pip, is installed.
pub fn require_command(cmd: &str) -> Result<()> {
    match which::which(cmd) {
        Ok(_) => Ok(()),
        Err(_) => bail!("missing required command: {}", cmd),
    }
}

/// Writes the bytes to file; creates a new file if it doesn't exist;
/// truncates (i.e overwrites) the file if it already exists.
pub fn write_file(path: &Path, bytes: &[u8]) -> Result<()> {
    let mut file = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(path)?;

    file.write_all(bytes)?;

    let metadata = file.metadata()?;
    let mut permissions = metadata.permissions();
    permissions.set_mode(0o644);
    fs::set_permissions(path, permissions)?;

    Ok(())
}

pub struct ScriptOptions {
    /// Make the script executable. Defaults to true.
    make_exec: bool,
    /// Add shebang on first line. Defaults to true.
    shebang: bool,
    /// The program to put in shebang. Defaults to bash.
    prog: String,
    /// Create a link from this location to the script created.
    link: Option<PathBuf>,
}

impl ScriptOptions {
    pub fn new() -> Self {
        Self {
            make_exec: true,
            shebang: true,
            prog: "bash".to_string(),
            link: None,
        }
    }

    /// Creates a new executable script at path given the options.
    pub fn create(&self, path: &Path, lines: Vec<String>) -> Result<()> {
        let file = fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(path)?;

        let metadata = file.metadata()?;

        let mut script: Vec<String> = Vec::new();

        if self.shebang {
            script.push(format!("#!/usr/bin/env {}", self.prog));
        }

        script.extend(lines);
        let script = script.join("\n");

        let mut writer = LineWriter::new(file);
        writer.write_all(script.as_bytes())?;

        if self.make_exec {
            make_exec(path, &metadata)?;
        }

        if let Some(link) = &self.link {
            let _ = fs::remove_file(link);
            symlink(link, path)?;
        }

        Ok(())
    }
}

#[cfg(unix)]
pub fn symlink(original: &Path, link: &Path) -> Result<()> {
    std::os::unix::fs::symlink(original, link)?;
    Ok(())
}

#[cfg(windows)]
pub fn symlink(original: &Path, link: &Path) -> Result<()> {
    std::os::windows::fs::symlink_file(original, link)?;
    Ok(())
}

/// Decompress an archive (zip or tar.gz) at `path` to `outdir`.
pub fn decompress(path: &Path, outpath: &Path) -> Result<()> {
    let fname = match path.file_name() {
        Some(ext) => ext,
        None => bail!("decompress: unable to identify archive type"),
    };

    let fname = match fname.to_str() {
        Some(ext) => ext,
        None => bail!("decompress: unable to identify archive type"),
    };

    let file = fs::File::open(path)?;
    if fname.ends_with("zip") {
        let mut archive = zip::ZipArchive::new(file)?;
        archive.extract(outpath)?;
    } else if fname.ends_with("tar.gz") {
        let tar = flate2::read::GzDecoder::new(file);
        let mut archive = tar::Archive::new(tar);
        archive.unpack(outpath)?;
    } else if fname.ends_with("gz") {
        let mut tar = flate2::read::GzDecoder::new(file);
        let mut buf: Vec<u8> = Vec::new();
        tar.read_to_end(&mut buf)?;
        write_file(outpath, &buf)?
    } else {
        bail!("decompress: unable to identify archive type");
    }

    Ok(())
}

#[allow(unused)]
pub fn make_executable(path: &Path) -> Result<()> {
    let file = fs::OpenOptions::new().read(true).open(path)?;
    let m = file.metadata()?;
    make_exec(path, &m)
}

fn make_exec(path: &Path, metadata: &fs::Metadata) -> Result<()> {
    let mut permissions = metadata.permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions)?;
    Ok(())
}

/// Returns a command that has stderr and stdout
/// bound to the Stdio::null() writer.
///
/// TODO: add option for getting output from the command.
pub fn new_cmd<S>(cmd: S) -> process::Command
where
    S: AsRef<OsStr>,
{
    let mut cmd = process::Command::new(cmd);
    cmd.stderr(Stdio::null());
    cmd.stdout(Stdio::null());
    cmd
}

/// Runs a command and checks the result.
pub fn run_cmd(cmd: &mut process::Command) -> Result<()> {
    let output = cmd.output()?;
    if !output.status.success() {
        let prog = cmd.get_program();
        log::info!("Command {:?} failed with status {}", prog, output.status);
        bail!("command {:?} failed with status {}", prog, output.status);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_require_command() {
        let ok = require_command("cargo");
        assert!(ok.is_ok());
        let err = require_command("non_existing_executable_error");
        assert!(err.is_err());
    }

    #[test]
    fn test_write_file() -> Result<()> {
        let dir = tempfile::tempdir_in(".")?;
        let filepath = dir.path().join("file.txt");
        write_file(&filepath, b"Testing.")
    }
}
