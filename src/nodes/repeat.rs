use crate::node::Node;
use crate::blackboard::BlackBoard;
use crate::node::NodeResult;
pub struct Repeat {
    pub child: Box<dyn Node>
}

impl Node for Repeat {
    fn tick(&mut self, memory: &mut BlackBoard) -> NodeResult {
        match self.child.tick(memory) {
            NodeResult::Passed => NodeResult::Passed,
            _ => NodeResult::Running
        }
    }
}