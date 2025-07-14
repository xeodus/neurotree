use crate::node::Node;
use crate::blackboard::BlackBoard;
use crate::node::NodeResult;
pub struct Selector {
    pub children: Vec<Box<dyn Node>>
}

impl Selector {
    pub fn new(children: Vec<Box<dyn Node>>) -> Self {
        Self {
            children
        }
    }
}

impl Node for Selector {
    fn tick(&mut self, memory: &mut BlackBoard) -> NodeResult {
        for i in &mut self.children {
            let result = i.tick(memory);
            match result {
                NodeResult::Passed => return NodeResult::Passed,
                NodeResult::Running => return NodeResult::Running,
                NodeResult::Failed => continue
            }
        }
        NodeResult::Failed
    }
}