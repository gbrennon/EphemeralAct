#[cfg(test)]
#[path = "../../fakes/fake_run_act_port.rs"]
mod fake_run_act_port;

#[cfg(test)]
mod tests {
    use ephemeral_act::presentation::composition_root::CompositionRoot;
    use fake_run_act_port::FakeRunActPort;

    use super::*;

    #[test]
    fn compose_creates_app_with_fake_port() {
        let port = FakeRunActPort::new(true);
        let _app = CompositionRoot::compose(port);
    }
}
