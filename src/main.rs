use ephemeral_act::{infrastructure::Container, presentation::composition_root::CompositionRoot};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let use_case = Container::build();
    let app = CompositionRoot::compose(use_case);
    match app.cli.run() {
        Ok(()) => std::process::exit(0),
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}
