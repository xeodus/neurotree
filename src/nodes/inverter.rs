use crate::node::Node;
use crate::blackboard::BlackBoard;
use crate::node::NodeResult;
pub struct Inverter {
    child: Box<dyn Node>
}

impl Node for Inverter {
    fn tick(&mut self, memory: &mut BlackBoard) -> NodeResult {
        match self.child.tick(memory) {
            NodeResult::Passed => return NodeResult::Passed,
            NodeResult::Failed => return NodeResult::Failed,
            NodeResult::Running => return NodeResult::Running
        }
    }
}