fn main() -> Result<(), Box<dyn std::error::Error>> {
    let target = target_build_utils::TargetInfo::new().expect("Coudln't get target info");

    if std::env::var("PROFILE").unwrap() == "release" && target.target_os() == "windows" {
        let mut res = winres::WindowsResource::new();
        if cfg!(unix) {
            res.set_toolkit_path("/usr/x86_64-w64-mingw32/bin");
            res.set_ar_path("ar");
            res.set_windres_path("/usr/bin/x86_64-w64-mingw32-windres");
            res.append_rc_content("1 ICON \"assets/logo.ico\"");
            res.append_rc_content("2 ICON \"assets/logo16_dark.ico\"");
            res.append_rc_content("3 ICON \"assets/logo16_light.ico\"");
            res.append_rc_content("4 ICON \"assets/logo32_dark.ico\"");
            res.append_rc_content("5 ICON \"assets/logo32_light.ico\"");
            println!("BUILDING WINDOWS RELEASE FROM LINUX");
        } else {
            res.append_rc_content("1 ICON \"assets\\logo.ico\"");
            res.append_rc_content("2 ICON \"assets\\logo16_dark.ico\"");
            res.append_rc_content("3 ICON \"assets\\logo16_light.ico\"");
            res.append_rc_content("4 ICON \"assets\\logo32_dark.ico\"");
            res.append_rc_content("5 ICON \"assets\\logo32_light.ico\"");
        }

        res.set_language(0x0409)
            .set_manifest_file("assets/winmanifest.xml");

        if let Err(e) = res.compile() {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    }

    tonic_build::configure()
        .build_server(false)
        .compile(&["proto/ddstats.proto"], &["proto"])?;

    Ok(())
}
