use ephemeral_act::{
    core::ports::inbound::RunActUseCase,
    infrastructure::Container,
    presentation::composition_root::CompositionRoot,
};

#[test]
fn compose_creates_app_with_cli() {
    let use_case = Container::build();
    let app = CompositionRoot::compose(use_case);
    // Verify the app has a CLI that can be called.
    let _ = app.cli;
}

#[test]
fn compose_result_has_expected_structure() {
    fn _assert_use_case(_: impl RunActUseCase) {}
    let use_case = Container::build();
    let app = CompositionRoot::compose(use_case);
    _assert_use_case(Container::build());
    let _ = app;
}
