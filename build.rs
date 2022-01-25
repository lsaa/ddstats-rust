fn main() -> Result<(), Box<dyn std::error::Error>> {

    #[cfg(target_os = "windows")] {
        use winres::WindowsResource;

        WindowsResource::new()
            .append_rc_content("1 ICON \"logo.ico\"")
            .append_rc_content("2 ICON \"logo_dark.ico\"")
            .append_rc_content("3 ICON \"logo_light.ico\"")
            .compile()?;
    }

    tonic_build::configure()
        .build_server(false)
        .compile(&["proto/ddstats.proto"], &["proto"])?;

    Ok(())
}
