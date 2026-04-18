mod app;
mod llm;
mod mdformator;
mod ui;

fn main() -> eframe::Result<()> {
    // Optional: `mdtoview path/to/file.md`
    let initial_content = std::env::args().nth(1).and_then(|path| {
        std::fs::read_to_string(&path)
            .map_err(|e| eprintln!("Cannot read '{}': {e}", path))
            .ok()
    });

    app::run(initial_content)
}
