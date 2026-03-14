mod wam_manifest;
mod virtual_asset_translator;
mod file_type_definitions;
mod asset_manager;

mod prelude {
    pub use std::path::PathBuf;
    pub use std::path::Path;
    pub use serde::Deserialize;
    pub use std::rc::Rc;
    pub use super::*;
    pub use slotmap::SlotMap;
    pub use std::collections::HashMap;
    pub use wam_manifest::*;
    pub use virtual_asset_translator::*;
    pub use file_type_definitions::*;
    pub use crate::app::WimpyIO;
}

pub use asset_manager::*;
pub use wam_manifest::*;
pub use file_type_definitions::*;
