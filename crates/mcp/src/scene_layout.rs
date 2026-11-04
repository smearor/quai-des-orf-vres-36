//! `SceneLayout` - every drawn entity's mm+px position
//! (`scene_layout` reply).

use serde::Deserialize;
use serde::Serialize;

use crate::scene_entity::SceneEntity;

/// All drawn entities under the current view transform.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneLayout {
    /// Station + landmark entities in draw order.
    pub entities: Vec<SceneEntity>,
}
