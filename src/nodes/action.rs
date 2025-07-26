use crate::{blackboard::BlackBoard, 
    node::{Node, NodeResult}};

pub struct Action {
    pub action: fn(&mut BlackBoard) -> NodeResult,
    pub name: String
}

impl Action {
    pub fn new(name: String, action: fn(&mut BlackBoard) -> NodeResult) -> Self {
        Self { action, name}
    }
}

impl Node for Action {
    fn tick(&mut self, blackboard: &mut BlackBoard) -> NodeResult {
        (self.action)(blackboard)
    }

    fn reset(&mut self) { }

    fn get_name(&self) -> String {
        self.name.clone()
    }
}