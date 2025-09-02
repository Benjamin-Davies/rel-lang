use crate::{Node, Rc, node};

impl Node {
    pub fn eval(&self, variables: impl IntoIterator<Item = bool>) -> Option<bool> {
        eval(&self.inner, variables.into_iter())
    }
}

fn eval(node: &Rc<node::Inner>, variables: impl Iterator<Item = bool>) -> Option<bool> {
    let mut current_node = Rc::clone(node);
    let mut variables = variables.enumerate();
    loop {
        match &current_node.kind {
            node::Kind::True => return Some(true),
            node::Kind::False => return Some(false),
            node::Kind::NonTerminal {
                cache: _,
                level,
                then_child,
                else_child,
            } => {
                let (_, variable) = variables.find(|&(i, _)| i as u64 == *level)?;
                if variable {
                    current_node = Rc::clone(then_child);
                } else {
                    current_node = Rc::clone(else_child);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::Manager;

    #[test]
    fn test_eval() {
        let manager = Manager::new();
        let a = manager.get_or_insert(1, &manager.true_node(), &manager.false_node());
        let b = manager.get_or_insert(2, &manager.false_node(), &manager.true_node());
        let root = manager.get_or_insert(0, &a, &b);

        assert_eq!(root.eval([false, false, false]), Some(true));
        assert_eq!(root.eval([false, false, true]), Some(false));
        assert_eq!(root.eval([false, true, false]), Some(true));
        assert_eq!(root.eval([false, true, true]), Some(false));
        assert_eq!(root.eval([true, false, false]), Some(false));
        assert_eq!(root.eval([true, false, true]), Some(false));
        assert_eq!(root.eval([true, true, false]), Some(true));
        assert_eq!(root.eval([true, true, true]), Some(true));

        assert_eq!(root.eval([]), None);
        assert_eq!(root.eval([false; 2]), None);
        assert_eq!(root.eval([true; 2]), Some(true));
        assert_eq!(root.eval([false; 4]), Some(true));
    }
}
