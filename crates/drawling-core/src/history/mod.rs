use crate::layer::LayerStack;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub enum HistoryAction {
    LayerPixelsChanged {
        layer_id: Uuid,
        before_pixels: Vec<u8>,
        after_pixels: Vec<u8>,
    },
    LayerAdded {
        index: usize,
    },
    LayerRemoved {
        index: usize,
        removed_layer: Box<crate::layer::Layer>,
    },
    LayerReordered {
        from: usize,
        to: usize,
    },
}

#[derive(Debug, Clone)]
pub struct HistoryStack {
    undo_stack: Vec<HistoryAction>,
    redo_stack: Vec<HistoryAction>,
    max_depth: usize,
}

impl HistoryStack {
    pub fn new(max_depth: usize) -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            max_depth: max_depth.max(5),
        }
    }

    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    pub fn push(&mut self, action: HistoryAction) {
        self.redo_stack.clear();
        self.undo_stack.push(action);
        if self.undo_stack.len() > self.max_depth {
            self.undo_stack.remove(0);
        }
    }

    pub fn undo(&mut self, stack: &mut LayerStack) -> bool {
        let action = match self.undo_stack.pop() {
            Some(a) => a,
            None => return false,
        };

        match &action {
            HistoryAction::LayerPixelsChanged {
                layer_id,
                before_pixels,
                ..
            } => {
                if let Some(l) = stack.layers.iter_mut().find(|l| l.id == *layer_id) {
                    l.pixels.copy_from_slice(before_pixels);
                }
            }
            HistoryAction::LayerAdded { index } => {
                stack.remove_layer(*index);
            }
            HistoryAction::LayerRemoved {
                index,
                removed_layer,
            } => {
                let idx = (*index).min(stack.layers.len());
                stack.layers.insert(idx, (**removed_layer).clone());
                stack.active_index = idx;
            }
            HistoryAction::LayerReordered { from, to } => {
                stack.reorder_layer(*to, *from);
            }
        }

        self.redo_stack.push(action);
        true
    }

    pub fn redo(&mut self, stack: &mut LayerStack) -> bool {
        let action = match self.redo_stack.pop() {
            Some(a) => a,
            None => return false,
        };

        match &action {
            HistoryAction::LayerPixelsChanged {
                layer_id,
                after_pixels,
                ..
            } => {
                if let Some(l) = stack.layers.iter_mut().find(|l| l.id == *layer_id) {
                    l.pixels.copy_from_slice(after_pixels);
                }
            }
            HistoryAction::LayerAdded { index } => {
                stack.add_layer(format!("Layer {}", index + 1));
            }
            HistoryAction::LayerRemoved { index, .. } => {
                stack.remove_layer(*index);
            }
            HistoryAction::LayerReordered { from, to } => {
                stack.reorder_layer(*from, *to);
            }
        }

        self.undo_stack.push(action);
        true
    }

    pub fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_history_undo_redo_pixels() {
        let mut stack = LayerStack::new(2, 2);
        let layer_id = stack.layers[1].id;
        let before_pixels = stack.layers[1].pixels.clone();

        // Simulate drawing
        stack.layers[1].set_pixel(0, 0, [255, 0, 0, 255]);
        let after_pixels = stack.layers[1].pixels.clone();

        let mut history = HistoryStack::new(10);
        assert!(!history.can_undo());
        assert!(!history.can_redo());

        history.push(HistoryAction::LayerPixelsChanged {
            layer_id,
            before_pixels: before_pixels.clone(),
            after_pixels: after_pixels.clone(),
        });

        assert!(history.can_undo());

        // Undo
        assert!(history.undo(&mut stack));
        assert_eq!(stack.layers[1].pixels, before_pixels);
        assert!(history.can_redo());

        // Redo
        assert!(history.redo(&mut stack));
        assert_eq!(stack.layers[1].pixels, after_pixels);
    }
}
