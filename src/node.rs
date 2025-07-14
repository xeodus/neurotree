use crate::blackboard::BlackBoard;

#[derive(Debug, Clone, PartialEq)]
pub enum NodeResult {
    Passed,
    Failed,
    Running
}

pub trait Node {
    fn tick(&mut self, memory: &mut BlackBoard) -> NodeResult;
}