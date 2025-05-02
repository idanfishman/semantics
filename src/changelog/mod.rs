pub mod config;
mod preset;
mod rule;

use std::collections::HashMap;

use anyhow::{Context, Result};
use git2::Commit;
use serde::Serialize;

use crate::changelog::config::ChangelogGeneratorConfig;
use crate::changelog::preset::Preset;
use crate::changelog::rule::{Category, Rule};
