#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]
use eframe::{self, NativeOptions, CreationContext, IconData};
use log::info;
use env_logger;
use anyhow::Result;

mod app;
mod ui;
mod parser;
mod model;
mod labelcodes;
mod export;

fn main() -> Result<()> {
    env_logger::init();
    info!("GEMA_Launcher startet");

    //src weg machen im path
    let image = image::open("src/assets/logo.png").expect("Kann 'logo.png' nicht öffnen");
    let image = image.to_rgba8();
    let (width, height) = image.dimensions();
    let icon_data = IconData {
        rgba: image.into_raw(),
        width,
        height,
    };

    let native_options = NativeOptions {
        icon_data: Some(icon_data),
        ..Default::default()
    };

    eframe::run_native(
        "GEMA_Launcher",
        native_options,
        Box::new(|_cc: &CreationContext| Box::new(app::GemaLauncherApp::default())),
    ).expect("GEMA_Launcher konnte nicht gestartet werden");

    Ok(())
}
