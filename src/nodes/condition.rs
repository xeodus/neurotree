use crate::{blackboard::BlackBoard, node::{Node, NodeResult}};

pub struct Condition {
    pub condition: fn(&mut BlackBoard) -> bool,
    pub is_key_present: bool
}

impl Condition {
    pub fn new(condition: fn(&mut BlackBoard) -> bool) -> Self {
        Self {
            condition,
            is_key_present: false
        }
    }
}

impl Node for Condition {
    fn tick(&mut self, blackboard: &mut BlackBoard) -> NodeResult {
        if (self.condition)(blackboard) && self.is_key_present {
            NodeResult::Passed
        }
        else {
            NodeResult::Failed
        }
    }
}