mod arch;

use std::ffi::OsString;
use bzip2::read::BzDecoder;
use color_eyre::Result;
use reqwest::Url;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use color_eyre::eyre::eyre;
use tar::Archive;
use tempdir::TempDir;
use trauma::download::Download;
use trauma::downloader::DownloaderBuilder;
use serde::Deserialize;
use shellexpand::tilde;
use lazy_static::lazy_static;
use arch::get_mamba_url;

lazy_static! {
    static ref MAMBA_PATH: PathBuf = PathBuf::from(tilde("~/micromamba").into_owned());
}

async fn download_tarbz2(url: Url, dest_dir: PathBuf) -> Result<()> {
    let temp_dir = TempDir::new("")?;
    let filename = "archive.tar.bz2";
    let downloads = vec![Download::new(&url, filename)];
    let downloader = DownloaderBuilder::new()
        .directory(temp_dir.path().to_path_buf())
        .build();
    downloader.download(&downloads).await;

    let archive_path = temp_dir.path().join(filename);
    let archive_file = File::open(archive_path)?;
    let mut bz2_decoder = BzDecoder::new(archive_file);
    let mut tar_data = Vec::new();
    bz2_decoder.read_to_end(&mut tar_data)?;

    let mut archive = Archive::new(&tar_data[..]);
    archive.unpack(dest_dir)?;
    Ok(())
}

#[derive(Deserialize)]
struct MambaEnvs {
    envs: Vec<String>,
}

impl MambaEnvs {
    fn load() -> Result<Self> {
        let output = std::process::Command::new(get_mamba_bin_path_expect()?)
            .arg("env")
            .arg("list")
            .arg("--json")
            .output()?
            .stdout;
        serde_json::from_slice(&output).map_err(|e| eyre!(e))
    }
}

fn which(bin_name: &str) -> Result<Option<PathBuf>> {
    let output = std::process::Command::new("which")
        .arg(bin_name)
        .output()?;
    if output.status.code() != Some(0) {
        return Ok(None);
    }
    let path = PathBuf::from(String::from_utf8(output.stdout)?.trim());
    if !path.exists() {
        return Err(eyre!("which successful but path does not exist"));
    }
    Ok(Some(path))
}

fn get_mamba_bin_path() -> Result<Option<OsString>> {
    let from_path = which("micromamba")?;
    if let Some(path) = from_path {
        return Ok(Some(path.into_os_string()));
    }
    let from_mamba_path = MAMBA_PATH.join("bin").join("micromamba");
    if from_mamba_path.exists() {
        return Ok(Some(from_mamba_path.into_os_string()));
    }
    Ok(None)
}

fn get_mamba_bin_path_expect() -> Result<OsString> {
    get_mamba_bin_path()?.ok_or_else(|| eyre!("failed to get mamba bin path"))
}

async fn ensure_installed() -> Result<()> {
    if get_mamba_bin_path()?.is_some() {
        return Ok(());
    }
    let mamba_url = get_mamba_url().ok_or_else(|| eyre!("failed to get mamba url"))?;
    download_tarbz2(
        Url::parse(&mamba_url).unwrap(),
        MAMBA_PATH.clone(),
    ).await?;
    Ok(())
}

fn has_env() -> Result<bool> {
    let envs = MambaEnvs::load()?;
    Ok(envs.envs.iter().map(|env| {
        PathBuf::from(env).exists()
    }).any(|x| x))
}

fn ensure_env() -> Result<()> {
    if has_env()? {
        return Ok(());
    }
    if !std::process::Command::new(
        get_mamba_bin_path()?.ok_or_else(|| eyre!("failed to get mamba bin path"))?
    )
        .arg("create")
        .arg("-n")
        .arg("base")
        .status()?.success() {
        Err(eyre!("failed to create base env"))
    } else {
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    ensure_installed().await?;
    ensure_env()?;
    let args : Vec<OsString> = std::env::args_os().skip(1).collect();
    let cmd = std::process::Command::new(get_mamba_bin_path_expect()?)
        .args(args)
        .status()?;
    if !cmd.success() {
        std::process::exit(cmd.code().unwrap_or(1));
    }
    Ok(())
}
