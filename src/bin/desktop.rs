use nexus::desktop::DesktopApp;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Nexus Desktop",
        options,
        Box::new(|_cc| Box::new(DesktopApp::new())),
    )
}
