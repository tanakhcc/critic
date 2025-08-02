//! Shared Types and functions accross the app

use leptos::prelude::*;
use leptos_router::params::Params;

#[derive(Params, Clone, PartialEq)]
pub struct MsParams {
    pub msname: Option<String>,
}

#[derive(Params, Clone, PartialEq)]
pub struct PageParams {
    pub pagename: Option<String>,
}
