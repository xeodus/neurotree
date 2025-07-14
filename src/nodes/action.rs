use crate::{blackboard::BlackBoard, node::NodeResult};

pub struct Action {
    pub action: fn(&mut BlackBoard) -> NodeResult
}

impl Action {
    pub fn new(&self, blackboard: &mut BlackBoard) -> NodeResult {
        (self.action)(blackboard)
    }
}