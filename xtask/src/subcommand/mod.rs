use std::collections::BTreeMap;

use serde::Deserialize;

pub mod build;
pub mod clean;
pub mod fmt;
pub mod r#move;
pub mod new;

#[derive(Deserialize)]
pub struct Config {
    pub domains: BTreeMap<String, Vec<String>>,
}
static DOMAIN_SET: [&str; 3] = ["common", "fs", "drivers"];
