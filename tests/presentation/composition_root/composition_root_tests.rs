mod common;

use ephemeral_act::presentation::composition_root::CompositionRoot;

use crate::common::FakeRunActUseCase;

#[test]
fn compose_creates_app_with_fake_use_case() {
    let use_case = FakeRunActUseCase::new(true);
    let _app = CompositionRoot::compose(use_case);
}
