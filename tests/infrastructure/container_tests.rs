use ephemeral_act::{
    core::ports::inbound::RunActUseCase,
    infrastructure::{Container, runners::ContainerRuntimeAdapter},
};

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! require_container_runtime {
        () => {
            if ContainerRuntimeAdapter::detect().is_err() {
                eprintln!("SKIP: no container runtime available (Docker or Podman required)");
                return;
            }
        };
    }

    #[test]
    fn build_returns_use_case_when_runtime_available() {
        require_container_runtime!();
        let _use_case = Container::build();
    }

    #[test]
    fn build_result_implements_run_act_use_case() {
        require_container_runtime!();
        fn _assert(_: impl RunActUseCase) {}
        _assert(Container::build());
    }
}
