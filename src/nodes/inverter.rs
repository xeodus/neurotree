use crate::node::Node;
use crate::blackboard::BlackBoard;
use crate::node::NodeResult;

pub struct Inverter {
    pub child: Box<dyn Node>,
    pub name: String
}

impl Inverter {
    pub fn new(child: Box<dyn Node>, name: String) -> Self {
        Self { child, name }
    }
}

impl Node for Inverter {
    fn tick(&mut self, memory: &mut BlackBoard) -> NodeResult {
        match self.child.tick(memory) {
            NodeResult::Passed => return NodeResult::Failed,
            NodeResult::Failed => return NodeResult::Passed,
            NodeResult::Running => return NodeResult::Running
        }
    }

    fn get_name(&self) -> String {
        self.name.clone()
    }

    fn reset(&mut self) {
        self.child.reset();
    }
}