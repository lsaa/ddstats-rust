fn main() -> Result<(), Box<dyn std::error::Error>> {

    #[cfg(target_os = "windows")] {
        use winres::WindowsResource;

        WindowsResource::new()
            .append_rc_content("1 ICON \"assets\\logo.ico\"")
            .append_rc_content("2 ICON \"assets\\logo16_dark.ico\"")
            .append_rc_content("3 ICON \"assets\\logo16_light.ico\"")
            .append_rc_content("4 ICON \"assets\\logo32_dark.ico\"")
            .append_rc_content("5 ICON \"assets\\logo32_light.ico\"")
            .compile()?;
    }

    tonic_build::configure()
        .build_server(false)
        .compile(&["proto/ddstats.proto"], &["proto"])?;

    Ok(())
}
