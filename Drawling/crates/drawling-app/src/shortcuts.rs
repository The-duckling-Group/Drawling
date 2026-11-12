#![allow(dead_code)]
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Action {
    ToggleZenMode,
    Undo,
    Redo,
    ToolBrush,
    ToolEraser,
    ToolEyedropper,
    ToolBucket,
    ToolLasso,
    ToolMagicWand,
    ToolTransform,
    SaveProject,
    ExportImage,
    ZoomIn,
    ZoomOut,
    ResetView,
    NewLayer,
    DeleteLayer,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShortcutManager {
    bindings: HashMap<String, Action>,
}

impl Default for ShortcutManager {
    fn default() -> Self {
        let mut bindings = HashMap::new();
        bindings.insert("Tab".into(), Action::ToggleZenMode);
        bindings.insert("F11".into(), Action::ToggleZenMode);
        bindings.insert("Ctrl+Z".into(), Action::Undo);
        bindings.insert("Ctrl+Y".into(), Action::Redo);
        bindings.insert("Ctrl+Shift+Z".into(), Action::Redo);
        bindings.insert("KeyB".into(), Action::ToolBrush);
        bindings.insert("KeyE".into(), Action::ToolEraser);
        bindings.insert("KeyI".into(), Action::ToolEyedropper);
        bindings.insert("KeyG".into(), Action::ToolBucket);
        bindings.insert("KeyL".into(), Action::ToolLasso);
        bindings.insert("KeyW".into(), Action::ToolMagicWand);
        bindings.insert("Ctrl+T".into(), Action::ToolTransform);
        bindings.insert("Ctrl+S".into(), Action::SaveProject);
        bindings.insert("Ctrl+E".into(), Action::ExportImage);
        bindings.insert("Ctrl+Plus".into(), Action::ZoomIn);
        bindings.insert("Ctrl+Minus".into(), Action::ZoomOut);
        bindings.insert("Ctrl+0".into(), Action::ResetView);
        bindings.insert("Ctrl+Shift+N".into(), Action::NewLayer);
        bindings.insert("Delete".into(), Action::DeleteLayer);

        Self { bindings }
    }
}

impl ShortcutManager {
    pub fn get_action(&self, key_combination: &str) -> Option<Action> {
        self.bindings.get(key_combination).copied()
    }

    pub fn set_binding(&mut self, key_combination: impl Into<String>, action: Action) {
        self.bindings.insert(key_combination.into(), action);
    }
}
