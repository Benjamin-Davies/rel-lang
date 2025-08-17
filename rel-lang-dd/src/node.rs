use alloc::rc::Rc;

use crate::manager::Cache;

#[derive(Debug)]
pub struct Node {
    pub(crate) cache: Rc<Cache>,
    pub(crate) kind: NodeKind,
}

#[derive(Debug)]
pub(crate) enum NodeKind {
    True,
    False,
    NonTerminal {
        then_child: Rc<Node>,
        else_child: Rc<Node>,
    },
}

impl Drop for Node {
    fn drop(&mut self) {
        match &self.kind {
            NodeKind::True | NodeKind::False => {
                // Do nothing
            }
            NodeKind::NonTerminal {
                then_child,
                else_child,
            } => {
                self.cache.remove(then_child, else_child);
            }
        }
    }
}
