use ephemeral_act::{infrastructure::Container, presentation::composition_root::CompositionRoot};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let use_case = Container::build();
    let app = CompositionRoot::compose(use_case);
    app.cli.run()
}
