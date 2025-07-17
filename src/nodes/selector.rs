use crate::node::Node;
use crate::blackboard::BlackBoard;
use crate::node::NodeResult;
pub struct Selector {
    pub children: Vec<Box<dyn Node>>,
    pub current_child: i32,
    pub name: String,
    pub is_running: bool
}

impl Selector {
    pub fn new(name: String, children: Vec<Box<dyn Node>>) -> Self {
        Self {
            children,
            current_child: 0,
            name,
            is_running: false
        }
    }
}

impl Node for Selector {
    fn tick(&mut self, memory: &mut BlackBoard) -> NodeResult {
        for child in &mut self.children {
            let result = child.tick(memory);
            match result {
                NodeResult::Passed => {
                    child.reset();
                    return NodeResult::Passed
                },
                NodeResult::Running => {
                    child.reset();
                    return NodeResult::Running
                },
                NodeResult::Failed => {
                    self.current_child += 1;
                    continue
                }
            }
        }
        NodeResult::Failed
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