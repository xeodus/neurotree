use crate::node::{Node, NodeResult};
use crate::blackboard::BlackBoard;
pub struct Sequence {
    pub children: Vec<Box<dyn Node>>,
    pub current_child: i32,
    pub name: String,
    pub is_running: bool
}

impl Sequence {
    pub fn new(name: String, children: Vec<Box<dyn Node>>) -> Self {
        Self {
            children,
            current_child: 0,
            name, 
            is_running: false
        }
    }
}

impl Node for Sequence {
    fn tick(&mut self, memory: &mut BlackBoard) -> NodeResult {
        for child in &mut self.children {
            let result = child.tick(memory);
            match result {
                NodeResult::Failed => return NodeResult::Failed,
                NodeResult::Running => return NodeResult::Running,
                NodeResult::Passed => {
                    self.current_child += 1;
                    continue;
                }
            }
        }
        NodeResult::Passed
    }

    fn get_name(&self) -> String {
        self.name.clone()
    }

    fn reset(&mut self) {
        self.current_child = 0;
        self.is_running = false;

        for child in &mut self.children {
            child.reset();
        }
    }
}