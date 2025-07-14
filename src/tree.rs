use crate::{blackboard::BlackBoard, node::{Node, NodeResult}};

pub struct BehaviouralTree {
    pub root: Box<dyn Node>,
    pub blackboard: BlackBoard
}

impl BehaviouralTree {
    pub fn new(root: Box<dyn Node>, blackboard: BlackBoard) -> Self {
        Self { root, blackboard }
    }
    pub fn tick(&mut self, blackboard: &mut BlackBoard) -> NodeResult {
        self.root.tick(blackboard)
    }
}