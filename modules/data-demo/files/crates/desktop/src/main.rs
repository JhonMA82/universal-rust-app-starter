fn main() -> std::process::ExitCode {
    if let Err(error) = ui_app::run() {
        eprintln!("{error}");
        return std::process::ExitCode::FAILURE;
    }
    std::process::ExitCode::SUCCESS
}
