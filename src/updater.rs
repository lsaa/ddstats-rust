// fuckign windles

use anyhow::Result;
use self_update::cargo_crate_version;
#[cfg(target_os = "linux")] use std::os::linux::fs::MetadataExt;

#[cfg(target_os = "windows")]
pub fn update() -> Result<()> {
    let status = self_update::backends::github::Update::configure()
        .repo_owner("lsaa")
        .repo_name("ddstats-rust")
        .bin_name("ddstats-rust")
        .show_download_progress(true)
        .current_version(cargo_crate_version!())
        .build()?
        .update()?;
    if !status.updated() {
        return Err(anyhow::anyhow!("already up to date"));
    }
    Ok(())
}

#[cfg(target_os = "linux")]
pub fn update() -> Result<()> {
    use std::{process::Command, env::current_exe};
    use std::os::unix::prelude::MetadataExt;

    let exe = current_exe()?;
    let has_write = {
        let metadata = exe.metadata();
        println!("exe path: {:?}", exe.to_str());
        if metadata.is_err() {
            false
        } else {
            let metadata = metadata?;
            let file_owner = metadata.uid();
            let user_id = std::fs::metadata("/proc/self").map(|m| m.uid())?;
            let mode = metadata.st_mode();
            let (r_user, w_user) = (mode & 0o000400 == 0o000400, mode & 0o000200 == 0o000200);
            let (r_other, w_other) = (mode & 0o000004 == 0o000004, mode & 0o000002 == 0o000002);
            (r_other && w_other) || ((file_owner == user_id) && (r_user && w_user))
        }
    };

    let instpath = exe.to_str().unwrap();
    println!("write access: {}", has_write);

    // Download and extract executable

    if has_write {
        let status = self_update::backends::github::Update::configure()
            .repo_owner("lsaa")
            .repo_name("ddstats-rust")
            .bin_name("ddstats-rust")
            .show_download_progress(true)
            .current_version(cargo_crate_version!())
            .build()?
            .update()?;

        if !status.updated() {
            return Err(anyhow::anyhow!("already up to date"));
        }
    } else {
        let status = self_update::backends::github::Update::configure()
            .repo_owner("lsaa")
            .repo_name("ddstats-rust")
            .bin_name("ddstats-rust")
            .show_download_progress(true)
            .current_version(cargo_crate_version!())
            .bin_install_path("/tmp/ddstats-rust")
            .build()?
            .update()?;

        if !status.updated() {
            return Err(anyhow::anyhow!("already up to date"));
        }

        Command::new("sudo")
            .arg("install")
            .arg("-Dm775")
            .arg("/tmp/ddstats-rust")
            .arg("/usr/bin/ddstats-rust")
            .status()?;
    }

    Command::new("sudo")
        .arg("setcap")
        .arg("cap_sys_ptrace=eip")
        .arg(instpath)
        .status()?;

    Ok(())
}
