mod display_form;
pub(crate) mod submit_form;

pub(crate) use display_form::display_form;
pub(crate) use submit_form::submit_form;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default, Clone)]
pub(crate) struct FormValues {
    pub(crate) shortname: String,
    pub(crate) longurl: String,
    pub(crate) shortname_is_disabled: bool,
}
