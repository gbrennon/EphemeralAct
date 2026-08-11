#[cfg(test)]
#[path = "../../fakes/fake_run_act_use_case.rs"]
mod fake_run_act_use_case;

#[cfg(test)]
mod tests {
    use ephemeral_act::presentation::composition_root::CompositionRoot;
    use fake_run_act_use_case::FakeRunActUseCase;

    use super::*;

    #[test]
    fn compose_creates_app_with_fake_use_case() {
        let use_case = FakeRunActUseCase::new(true);
        let _app = CompositionRoot::compose(use_case);
    }
}
