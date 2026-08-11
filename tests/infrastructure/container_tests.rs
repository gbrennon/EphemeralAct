use ephemeral_act::{core::ports::inbound::RunActUseCase, infrastructure::Container};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_returns_use_case_when_runtime_available() {
        let _use_case = Container::build();
    }

    #[test]
    fn build_result_implements_run_act_use_case() {
        fn _assert(_: impl RunActUseCase) {}
        _assert(Container::build());
    }
}
