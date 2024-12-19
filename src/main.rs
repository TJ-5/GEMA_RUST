use eframe::{NativeOptions, CreationContext};
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

    let native_options = NativeOptions::default();
    eframe::run_native(
        "GEMA_Launcher",
        native_options,
        Box::new(|_cc: &CreationContext| Box::new(app::GemaLauncherApp::default())),
    ).expect("GEMA_Launcher konnte nicht gestartet werden");

    Ok(())
}
