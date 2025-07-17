// Working Implementation Example: Enhanced NeuroTree
// This demonstrates how to implement the key missing components

use std::collections::HashMap;
use std::time::{Duration, Instant};
use std::any::Any;
use std::error::Error;

// ===============================
// CORE TRAITS AND ENUMS
// ===============================

#[derive(Debug, Clone, PartialEq)]
pub enum NodeResult {
    Passed,
    Failed,
    Running,
}

// Enhanced Node trait with reset and name methods
pub trait Node: Send + Sync {
    fn tick(&mut self, blackboard: &mut BlackBoard) -> NodeResult;
    fn reset(&mut self);
    fn name(&self) -> &str;
}

// Enhanced BlackBoard with better error handling
pub struct BlackBoard {
    data: HashMap<String, Box<dyn Any + Send + Sync>>,
}

impl BlackBoard {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    pub fn set<T: 'static + Send + Sync>(&mut self, key: &str, value: T) {
        self.data.insert(key.to_string(), Box::new(value));
    }

    pub fn get<T: 'static>(&self, key: &str) -> Option<&T> {
        self.data.get(key).and_then(|f| f.downcast_ref::<T>())
    }

    pub fn contains_key(&self, key: &str) -> bool {
        self.data.contains_key(key)
    }

    pub fn remove(&mut self, key: &str) -> bool {
        self.data.remove(key).is_some()
    }
}

// ===============================
// FIXED ACTION NODE
// ===============================

pub struct Action {
    action: fn(&mut BlackBoard) -> NodeResult,
    name: String,
}

impl Action {
    pub fn new(name: &str, action: fn(&mut BlackBoard) -> NodeResult) -> Self {
        Self {
            action,
            name: name.to_string(),
        }
    }
}

impl Node for Action {
    fn tick(&mut self, blackboard: &mut BlackBoard) -> NodeResult {
        (self.action)(blackboard)
    }

    fn reset(&mut self) {
        // Actions don't maintain state, so nothing to reset
    }

    fn name(&self) -> &str {
        &self.name
    }
}

// ===============================
// STATEFUL COMPOSITE NODES
// ===============================

pub struct Sequence {
    children: Vec<Box<dyn Node>>,
    current_child: usize,
    name: String,
}

impl Sequence {
    pub fn new(name: &str, children: Vec<Box<dyn Node>>) -> Self {
        Self {
            children,
            current_child: 0,
            name: name.to_string(),
        }
    }
}

impl Node for Sequence {
    fn tick(&mut self, blackboard: &mut BlackBoard) -> NodeResult {
        while self.current_child < self.children.len() {
            let result = self.children[self.current_child].tick(blackboard);
            
            match result {
                NodeResult::Passed => {
                    self.current_child += 1;
                    continue;
                }
                NodeResult::Failed => {
                    self.reset();
                    return NodeResult::Failed;
                }
                NodeResult::Running => {
                    return NodeResult::Running;
                }
            }
        }
        
        // All children passed
        self.reset();
        NodeResult::Passed
    }

    fn reset(&mut self) {
        self.current_child = 0;
        for child in &mut self.children {
            child.reset();
        }
    }

    fn name(&self) -> &str {
        &self.name
    }
}

pub struct Selector {
    children: Vec<Box<dyn Node>>,
    current_child: usize,
    name: String,
}

impl Selector {
    pub fn new(name: &str, children: Vec<Box<dyn Node>>) -> Self {
        Self {
            children,
            current_child: 0,
            name: name.to_string(),
        }
    }
}

impl Node for Selector {
    fn tick(&mut self, blackboard: &mut BlackBoard) -> NodeResult {
        while self.current_child < self.children.len() {
            let result = self.children[self.current_child].tick(blackboard);
            
            match result {
                NodeResult::Passed => {
                    self.reset();
                    return NodeResult::Passed;
                }
                NodeResult::Failed => {
                    self.current_child += 1;
                    continue;
                }
                NodeResult::Running => {
                    return NodeResult::Running;
                }
            }
        }
        
        // All children failed
        self.reset();
        NodeResult::Failed
    }

    fn reset(&mut self) {
        self.current_child = 0;
        for child in &mut self.children {
            child.reset();
        }
    }

    fn name(&self) -> &str {
        &self.name
    }
}

// ===============================
// PARALLEL NODE
// ===============================

#[derive(Debug, Clone)]
pub enum ParallelPolicy {
    RequireAll,
    RequireOne,
    RequireCount(usize),
}

pub struct Parallel {
    children: Vec<Box<dyn Node>>,
    child_states: Vec<NodeResult>,
    policy: ParallelPolicy,
    name: String,
}

impl Parallel {
    pub fn new(name: &str, children: Vec<Box<dyn Node>>, policy: ParallelPolicy) -> Self {
        let child_count = children.len();
        Self {
            children,
            child_states: vec![NodeResult::Running; child_count],
            policy,
            name: name.to_string(),
        }
    }

    fn evaluate_policy(&self) -> NodeResult {
        let passed_count = self.child_states.iter()
            .filter(|&&state| state == NodeResult::Passed)
            .count();
        let failed_count = self.child_states.iter()
            .filter(|&&state| state == NodeResult::Failed)
            .count();

        match self.policy {
            ParallelPolicy::RequireAll => {
                if passed_count == self.children.len() {
                    NodeResult::Passed
                } else if failed_count > 0 {
                    NodeResult::Failed
                } else {
                    NodeResult::Running
                }
            }
            ParallelPolicy::RequireOne => {
                if passed_count > 0 {
                    NodeResult::Passed
                } else if failed_count == self.children.len() {
                    NodeResult::Failed
                } else {
                    NodeResult::Running
                }
            }
            ParallelPolicy::RequireCount(required) => {
                if passed_count >= required {
                    NodeResult::Passed
                } else if failed_count > self.children.len() - required {
                    NodeResult::Failed
                } else {
                    NodeResult::Running
                }
            }
        }
    }
}

impl Node for Parallel {
    fn tick(&mut self, blackboard: &mut BlackBoard) -> NodeResult {
        // Execute all children that are still running
        for (i, child) in self.children.iter_mut().enumerate() {
            if self.child_states[i] == NodeResult::Running {
                self.child_states[i] = child.tick(blackboard);
            }
        }

        self.evaluate_policy()
    }

    fn reset(&mut self) {
        for (i, child) in self.children.iter_mut().enumerate() {
            child.reset();
            self.child_states[i] = NodeResult::Running;
        }
    }

    fn name(&self) -> &str {
        &self.name
    }
}

// ===============================
// DECORATOR NODES
// ===============================

pub struct Inverter {
    child: Box<dyn Node>,
    name: String,
}

impl Inverter {
    pub fn new(name: &str, child: Box<dyn Node>) -> Self {
        Self {
            child,
            name: name.to_string(),
        }
    }
}

impl Node for Inverter {
    fn tick(&mut self, blackboard: &mut BlackBoard) -> NodeResult {
        match self.child.tick(blackboard) {
            NodeResult::Passed => NodeResult::Failed,
            NodeResult::Failed => NodeResult::Passed,
            NodeResult::Running => NodeResult::Running,
        }
    }

    fn reset(&mut self) {
        self.child.reset();
    }

    fn name(&self) -> &str {
        &self.name
    }
}

pub struct Timeout {
    child: Box<dyn Node>,
    timeout_duration: Duration,
    start_time: Option<Instant>,
    name: String,
}

impl Timeout {
    pub fn new(name: &str, child: Box<dyn Node>, timeout_duration: Duration) -> Self {
        Self {
            child,
            timeout_duration,
            start_time: None,
            name: name.to_string(),
        }
    }
}

impl Node for Timeout {
    fn tick(&mut self, blackboard: &mut BlackBoard) -> NodeResult {
        // Start timer on first tick
        if self.start_time.is_none() {
            self.start_time = Some(Instant::now());
        }

        // Check if timeout exceeded
        if let Some(start) = self.start_time {
            if start.elapsed() > self.timeout_duration {
                self.reset();
                return NodeResult::Failed;
            }
        }

        let result = self.child.tick(blackboard);

        // Reset timer if child completes
        if result != NodeResult::Running {
            self.start_time = None;
        }

        result
    }

    fn reset(&mut self) {
        self.child.reset();
        self.start_time = None;
    }

    fn name(&self) -> &str {
        &self.name
    }
}

// ===============================
// BUILDER PATTERN
// ===============================

pub struct TreeBuilder;

impl TreeBuilder {
    pub fn new() -> Self {
        Self
    }

    pub fn action(name: &str, action: fn(&mut BlackBoard) -> NodeResult) -> Box<dyn Node> {
        Box::new(Action::new(name, action))
    }

    pub fn sequence(name: &str) -> SequenceBuilder {
        SequenceBuilder::new(name)
    }

    pub fn selector(name: &str) -> SelectorBuilder {
        SelectorBuilder::new(name)
    }

    pub fn parallel(name: &str, policy: ParallelPolicy) -> ParallelBuilder {
        ParallelBuilder::new(name, policy)
    }

    pub fn inverter(name: &str, child: Box<dyn Node>) -> Box<dyn Node> {
        Box::new(Inverter::new(name, child))
    }

    pub fn timeout(name: &str, child: Box<dyn Node>, duration: Duration) -> Box<dyn Node> {
        Box::new(Timeout::new(name, child, duration))
    }
}

pub struct SequenceBuilder {
    name: String,
    children: Vec<Box<dyn Node>>,
}

impl SequenceBuilder {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            children: Vec::new(),
        }
    }

    pub fn child(mut self, child: Box<dyn Node>) -> Self {
        self.children.push(child);
        self
    }

    pub fn build(self) -> Box<dyn Node> {
        Box::new(Sequence::new(&self.name, self.children))
    }
}

pub struct SelectorBuilder {
    name: String,
    children: Vec<Box<dyn Node>>,
}

impl SelectorBuilder {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            children: Vec::new(),
        }
    }

    pub fn child(mut self, child: Box<dyn Node>) -> Self {
        self.children.push(child);
        self
    }

    pub fn build(self) -> Box<dyn Node> {
        Box::new(Selector::new(&self.name, self.children))
    }
}

pub struct ParallelBuilder {
    name: String,
    policy: ParallelPolicy,
    children: Vec<Box<dyn Node>>,
}

impl ParallelBuilder {
    pub fn new(name: &str, policy: ParallelPolicy) -> Self {
        Self {
            name: name.to_string(),
            policy,
            children: Vec::new(),
        }
    }

    pub fn child(mut self, child: Box<dyn Node>) -> Self {
        self.children.push(child);
        self
    }

    pub fn build(self) -> Box<dyn Node> {
        Box::new(Parallel::new(&self.name, self.children, self.policy))
    }
}

// ===============================
// BEHAVIORAL TREE
// ===============================

pub struct BehavioralTree {
    root: Box<dyn Node>,
    blackboard: BlackBoard,
}

impl BehavioralTree {
    pub fn new(root: Box<dyn Node>) -> Self {
        Self {
            root,
            blackboard: BlackBoard::new(),
        }
    }

    pub fn tick(&mut self) -> NodeResult {
        self.root.tick(&mut self.blackboard)
    }

    pub fn reset(&mut self) {
        self.root.reset();
    }

    pub fn blackboard(&mut self) -> &mut BlackBoard {
        &mut self.blackboard
    }
}

// ===============================
// ROBOTICS EXAMPLE USAGE
// ===============================

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    // Example robot state
    #[derive(Debug, Clone)]
    struct RobotState {
        position: (f64, f64),
        battery_level: f64,
        obstacle_detected: bool,
    }

    // Mock robot functions
    fn check_battery(blackboard: &mut BlackBoard) -> NodeResult {
        if let Some(state) = blackboard.get::<RobotState>("robot_state") {
            if state.battery_level > 0.2 {
                NodeResult::Passed
            } else {
                NodeResult::Failed
            }
        } else {
            NodeResult::Failed
        }
    }

    fn scan_environment(blackboard: &mut BlackBoard) -> NodeResult {
        // Simulate scanning
        blackboard.set("scan_complete", true);
        NodeResult::Passed
    }

    fn move_forward(blackboard: &mut BlackBoard) -> NodeResult {
        if let Some(mut state) = blackboard.get::<RobotState>("robot_state").cloned() {
            state.position.0 += 1.0;
            blackboard.set("robot_state", state);
            NodeResult::Passed
        } else {
            NodeResult::Failed
        }
    }

    fn avoid_obstacle(blackboard: &mut BlackBoard) -> NodeResult {
        if let Some(mut state) = blackboard.get::<RobotState>("robot_state").cloned() {
            state.position.1 += 1.0; // Move sideways
            blackboard.set("robot_state", state);
            NodeResult::Passed
        } else {
            NodeResult::Failed
        }
    }

    #[test]
    fn test_robot_behavior_tree() {
        let mut tree = BehavioralTree::new(
            TreeBuilder::selector("root")
                .child(
                    TreeBuilder::sequence("normal_operation")
                        .child(TreeBuilder::action("check_battery", check_battery))
                        .child(TreeBuilder::action("scan", scan_environment))
                        .child(TreeBuilder::action("move", move_forward))
                        .build()
                )
                .child(TreeBuilder::action("avoid_obstacle", avoid_obstacle))
                .build()
        );

        // Set initial robot state
        tree.blackboard().set("robot_state", RobotState {
            position: (0.0, 0.0),
            battery_level: 0.8,
            obstacle_detected: false,
        });

        // Run the tree
        let result = tree.tick();
        assert_eq!(result, NodeResult::Passed);

        // Check if robot moved
        let state = tree.blackboard().get::<RobotState>("robot_state").unwrap();
        assert_eq!(state.position.0, 1.0);
    }

    #[test]
    fn test_parallel_execution() {
        let mut tree = BehavioralTree::new(
            TreeBuilder::parallel("parallel_test", ParallelPolicy::RequireAll)
                .child(TreeBuilder::action("action1", |_| NodeResult::Passed))
                .child(TreeBuilder::action("action2", |_| NodeResult::Passed))
                .build()
        );

        let result = tree.tick();
        assert_eq!(result, NodeResult::Passed);
    }

    #[test]
    fn test_timeout_decorator() {
        let mut tree = BehavioralTree::new(
            TreeBuilder::timeout(
                "timeout_test",
                TreeBuilder::action("long_action", |_| NodeResult::Running),
                Duration::from_millis(100)
            )
        );

        // First tick should return Running
        let result1 = tree.tick();
        assert_eq!(result1, NodeResult::Running);

        // Wait for timeout
        std::thread::sleep(Duration::from_millis(150));

        // Second tick should return Failed due to timeout
        let result2 = tree.tick();
        assert_eq!(result2, NodeResult::Failed);
    }

    #[test]
    fn test_inverter_decorator() {
        let mut tree = BehavioralTree::new(
            TreeBuilder::inverter(
                "inverter_test",
                TreeBuilder::action("fail_action", |_| NodeResult::Failed)
            )
        );

        let result = tree.tick();
        assert_eq!(result, NodeResult::Passed); // Failed inverted to Passed
    }
}

// ===============================
// EXAMPLE MAIN FUNCTION
// ===============================

fn main() {
    // Example: Simple robot patrol behavior
    let mut patrol_tree = BehavioralTree::new(
        TreeBuilder::selector("patrol")
            .child(
                TreeBuilder::sequence("normal_patrol")
                    .child(TreeBuilder::action("check_battery", check_battery))
                    .child(TreeBuilder::action("scan_area", scan_environment))
                    .child(TreeBuilder::action("move_to_next_point", move_forward))
                    .build()
            )
            .child(TreeBuilder::action("return_to_base", |bb| {
                println!("Returning to base for charging");
                NodeResult::Passed
            }))
            .build()
    );

    // Initialize robot state
    patrol_tree.blackboard().set("robot_state", RobotState {
        position: (0.0, 0.0),
        battery_level: 0.8,
        obstacle_detected: false,
    });

    // Run patrol loop
    for i in 0..5 {
        println!("Patrol cycle {}", i + 1);
        let result = patrol_tree.tick();
        println!("Result: {:?}", result);
        
        // Simulate battery drain
        if let Some(mut state) = patrol_tree.blackboard().get::<RobotState>("robot_state").cloned() {
            state.battery_level -= 0.2;
            patrol_tree.blackboard().set("robot_state", state);
        }
    }
}

// Helper functions for main
fn check_battery(blackboard: &mut BlackBoard) -> NodeResult {
    if let Some(state) = blackboard.get::<RobotState>("robot_state") {
        if state.battery_level > 0.2 {
            println!("Battery OK: {:.1}%", state.battery_level * 100.0);
            NodeResult::Passed
        } else {
            println!("Battery low: {:.1}%", state.battery_level * 100.0);
            NodeResult::Failed
        }
    } else {
        NodeResult::Failed
    }
}

fn scan_environment(blackboard: &mut BlackBoard) -> NodeResult {
    println!("Scanning environment...");
    blackboard.set("scan_complete", true);
    NodeResult::Passed
}

fn move_forward(blackboard: &mut BlackBoard) -> NodeResult {
    if let Some(mut state) = blackboard.get::<RobotState>("robot_state").cloned() {
        state.position.0 += 1.0;
        blackboard.set("robot_state", state);
        println!("Moving to position: ({:.1}, {:.1})", state.position.0, state.position.1);
        NodeResult::Passed
    } else {
        NodeResult::Failed
    }
}

#[derive(Debug, Clone)]
struct RobotState {
    position: (f64, f64),
    battery_level: f64,
    obstacle_detected: bool,
}