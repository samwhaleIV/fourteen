use crate::wam::{WamManifest};

pub struct AssetManager {
    manifest: WamManifest,
}

impl AssetManager {
    pub fn create(manifest: WamManifest) -> Self {
        return Self {
            manifest
        }
    }

    //pub fn load_model<IO: WimpyIO>(man
}
