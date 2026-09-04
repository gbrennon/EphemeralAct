use std::path::Path;

use crate::{
    application::dtos::ExecuteActionRequest,
    domain::{expression::EvalContext, workflow::Step},
};

/// Everything needed to run a single step of a composite action.
pub struct RunCompositeStepRequest<'a> {
    pub step: &'a Step,

    pub action_dir: &'a Path,

    pub action_request: &'a ExecuteActionRequest,

    pub context: &'a EvalContext,
}
