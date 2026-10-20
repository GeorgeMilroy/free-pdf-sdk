//! Writes the synthetic PDF fixtures to tests/fixtures/ (for the e2e tests and the example).

fn main() -> std::process::ExitCode {
    let fixtures = match fps_tests::pdf_fixtures() {
        Ok(f) => f,
        Err(e) => {
            eprintln!("make-fixtures: {e}");
            return std::process::ExitCode::FAILURE;
        }
    };
    let dir = fps_tests::fixtures_dir();
    for (name, bytes) in fixtures {
        let path = dir.join(name);
        if let Err(e) = std::fs::write(&path, &bytes) {
            eprintln!("make-fixtures: {}: {e}", path.display());
            return std::process::ExitCode::FAILURE;
        }
        println!("wrote {} ({} bytes)", path.display(), bytes.len());
    }
    std::process::ExitCode::SUCCESS
}
