use crate::node::{Node, NodeResult};
use crate::blackboard::BlackBoard;
pub struct Sequence {
    pub children: Vec<Box<dyn Node>>
}

impl Sequence {
    pub fn new(children: Vec<Box<dyn Node>>) -> Self {
        Self {
            children
        }
    }
}

impl Node for Sequence {
    fn tick(&mut self, memory: &mut BlackBoard) -> NodeResult {
        for i in &mut self.children {
            let result = i.tick(memory);
            match result {
                NodeResult::Failed => return NodeResult::Failed,
                NodeResult::Running => return NodeResult::Running,
                NodeResult::Passed => continue
            }
        }
        NodeResult::Passed
    }
}