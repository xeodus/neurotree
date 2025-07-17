pub mod node;
pub mod blackboard;
pub mod tree;
pub mod nodes;

#[cfg(test)]

mod tests {
    use crate::{blackboard::BlackBoard, node::{Node, NodeResult}};

    #[test]
    fn test_node_result_variants() {
        assert_ne!(NodeResult::Passed, NodeResult::Failed);
        assert_ne!(NodeResult::Passed, NodeResult::Running);
        assert_ne!(NodeResult::Failed, NodeResult::Running)
    }

    pub struct TestNode {
        result: NodeResult
    }

    impl Node for TestNode {
        fn tick(&mut self, _blackboard: &mut BlackBoard) -> NodeResult {
            self.result.clone()
        }

        fn get_name(&self) -> String {
            "IS_ENEMY".into()
        }
        fn reset(&mut self) {
            
        }
    }

    #[test]
    fn test_mock_node() {
        let mut blackboard = BlackBoard::new();
        let mut node = TestNode { result: NodeResult::Passed };
        assert_eq!(node.tick(&mut blackboard), NodeResult::Passed);
        node.result = NodeResult::Failed;
        assert_eq!(node.tick(&mut blackboard), NodeResult::Failed);
        node.result = NodeResult::Running;
        assert_eq!(node.tick(&mut blackboard), NodeResult::Running);
    }
}