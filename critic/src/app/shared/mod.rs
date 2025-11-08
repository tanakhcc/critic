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

#[derive(Params, Clone, PartialEq)]
pub struct ModelParams {
    pub id: Option<i64>,
}

#[derive(Params, Clone, PartialEq)]
pub struct LanguageParams {
    pub language: Option<String>,
}
