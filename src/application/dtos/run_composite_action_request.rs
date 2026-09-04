use std::{collections::HashMap, path::Path};

use crate::{application::dtos::ExecuteActionRequest, domain::workflow::Step};

/// Everything needed to run the steps of a composite action.
pub struct RunCompositeActionRequest<'a> {
    pub steps: &'a [Step],

    pub inputs: &'a HashMap<String, String>,

    pub action_dir: &'a Path,

    pub action_request: &'a ExecuteActionRequest,
}
