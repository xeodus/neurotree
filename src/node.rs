use crate::blackboard::BlackBoard;

#[derive(Debug, Clone, PartialEq)]
pub enum NodeResult {
    Passed,
    Failed,
    Running
}

pub trait Node: Send + Sync {
    fn tick(&mut self, memory: &mut BlackBoard) -> NodeResult;
    fn reset(&mut self);
    fn get_name(&self) -> String;
}
