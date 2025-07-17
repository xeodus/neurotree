# NeuroTree: Detailed Implementation Guide & Design Patterns

## Part 1: Detailed Pseudo Code for Missing Components

### 1. Fixed Action Node Implementation

```rust
// Fixed Action Node with proper Node trait implementation
pub struct Action {
    action: fn(&mut BlackBoard) -> NodeResult,
    name: String,  // For debugging
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
        println!("Executing action: {}", self.name);
        (self.action)(blackboard)
    }
    
    fn reset(&mut self) {
        // Actions typically don't need reset, but interface requires it
    }
    
    fn name(&self) -> &str {
        &self.name
    }
}
```

### 2. Stateful Composite Nodes

```rust
// Enhanced Sequence with state tracking
pub struct Sequence {
    children: Vec<Box<dyn Node>>,
    current_child: usize,
    is_running: bool,
    name: String,
}

impl Sequence {
    pub fn new(name: &str, children: Vec<Box<dyn Node>>) -> Self {
        Self {
            children,
            current_child: 0,
            is_running: false,
            name: name.to_string(),
        }
    }
}

impl Node for Sequence {
    fn tick(&mut self, blackboard: &mut BlackBoard) -> NodeResult {
        // Resume from where we left off if we were running
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
                    self.is_running = true;
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
        self.is_running = false;
        for child in &mut self.children {
            child.reset();
        }
    }
    
    fn name(&self) -> &str {
        &self.name
    }
}

// Enhanced Selector with state tracking
pub struct Selector {
    children: Vec<Box<dyn Node>>,
    current_child: usize,
    is_running: bool,
    name: String,
}

impl Selector {
    pub fn new(name: &str, children: Vec<Box<dyn Node>>) -> Self {
        Self {
            children,
            current_child: 0,
            is_running: false,
            name: name.to_string(),
        }
    }
}

impl Node for Selector {
    fn tick(&mut self, blackboard: &mut BlackBoard) -> NodeResult {
        // Resume from where we left off if we were running
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
                    self.is_running = true;
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
        self.is_running = false;
        for child in &mut self.children {
            child.reset();
        }
    }
    
    fn name(&self) -> &str {
        &self.name
    }
}
```

### 3. Parallel Node Implementation

```rust
// Parallel node for concurrent execution
pub enum ParallelPolicy {
    RequireAll,    // All children must succeed
    RequireOne,    // At least one child must succeed
    RequireCount(usize), // Specific number must succeed
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
        let running_count = self.child_states.iter()
            .filter(|&&state| state == NodeResult::Running)
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
```

### 4. Advanced Decorator Nodes

```rust
// Timeout decorator
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

// Retry decorator
pub struct Retry {
    child: Box<dyn Node>,
    max_attempts: usize,
    current_attempt: usize,
    name: String,
}

impl Retry {
    pub fn new(name: &str, child: Box<dyn Node>, max_attempts: usize) -> Self {
        Self {
            child,
            max_attempts,
            current_attempt: 0,
            name: name.to_string(),
        }
    }
}

impl Node for Retry {
    fn tick(&mut self, blackboard: &mut BlackBoard) -> NodeResult {
        let result = self.child.tick(blackboard);
        
        match result {
            NodeResult::Passed => {
                self.current_attempt = 0;
                NodeResult::Passed
            }
            NodeResult::Failed => {
                self.current_attempt += 1;
                if self.current_attempt >= self.max_attempts {
                    self.current_attempt = 0;
                    NodeResult::Failed
                } else {
                    self.child.reset();
                    NodeResult::Running
                }
            }
            NodeResult::Running => NodeResult::Running,
        }
    }
    
    fn reset(&mut self) {
        self.child.reset();
        self.current_attempt = 0;
    }
    
    fn name(&self) -> &str {
        &self.name
    }
}

// Cooldown decorator
pub struct Cooldown {
    child: Box<dyn Node>,
    cooldown_duration: Duration,
    last_execution: Option<Instant>,
    name: String,
}

impl Cooldown {
    pub fn new(name: &str, child: Box<dyn Node>, cooldown_duration: Duration) -> Self {
        Self {
            child,
            cooldown_duration,
            last_execution: None,
            name: name.to_string(),
        }
    }
}

impl Node for Cooldown {
    fn tick(&mut self, blackboard: &mut BlackBoard) -> NodeResult {
        // Check if still in cooldown
        if let Some(last) = self.last_execution {
            if last.elapsed() < self.cooldown_duration {
                return NodeResult::Failed;
            }
        }
        
        let result = self.child.tick(blackboard);
        
        // Update last execution time when child completes
        if result != NodeResult::Running {
            self.last_execution = Some(Instant::now());
        }
        
        result
    }
    
    fn reset(&mut self) {
        self.child.reset();
        // Don't reset cooldown timer
    }
    
    fn name(&self) -> &str {
        &self.name
    }
}
```

### 5. Asynchronous Action Support

```rust
// Async action trait for long-running operations
pub trait AsyncAction: Send + Sync {
    fn start(&mut self, blackboard: &mut BlackBoard) -> Result<(), Box<dyn Error>>;
    fn update(&mut self, blackboard: &mut BlackBoard) -> NodeResult;
    fn stop(&mut self, blackboard: &mut BlackBoard);
    fn is_running(&self) -> bool;
}

// Async action wrapper
pub struct AsyncActionNode {
    action: Box<dyn AsyncAction>,
    state: AsyncActionState,
    name: String,
}

#[derive(Debug, Clone)]
enum AsyncActionState {
    NotStarted,
    Running,
    Completed(NodeResult),
}

impl AsyncActionNode {
    pub fn new(name: &str, action: Box<dyn AsyncAction>) -> Self {
        Self {
            action,
            state: AsyncActionState::NotStarted,
            name: name.to_string(),
        }
    }
}

impl Node for AsyncActionNode {
    fn tick(&mut self, blackboard: &mut BlackBoard) -> NodeResult {
        match self.state {
            AsyncActionState::NotStarted => {
                match self.action.start(blackboard) {
                    Ok(_) => {
                        self.state = AsyncActionState::Running;
                        NodeResult::Running
                    }
                    Err(_) => NodeResult::Failed,
                }
            }
            AsyncActionState::Running => {
                let result = self.action.update(blackboard);
                if result != NodeResult::Running {
                    self.state = AsyncActionState::Completed(result.clone());
                }
                result
            }
            AsyncActionState::Completed(result) => result,
        }
    }
    
    fn reset(&mut self) {
        if let AsyncActionState::Running = self.state {
            self.action.stop(blackboard);
        }
        self.state = AsyncActionState::NotStarted;
    }
    
    fn name(&self) -> &str {
        &self.name
    }
}
```

### 6. Robotics-Specific Implementations

```rust
// Sensor integration
pub trait SensorReader: Send + Sync {
    fn read(&mut self) -> Result<SensorData, Box<dyn Error>>;
    fn sensor_type(&self) -> &str;
}

pub struct SensorData {
    pub timestamp: Instant,
    pub data: HashMap<String, Box<dyn Any>>,
}

pub struct SensorNode {
    sensor: Box<dyn SensorReader>,
    blackboard_key: String,
    name: String,
}

impl SensorNode {
    pub fn new(name: &str, sensor: Box<dyn SensorReader>, blackboard_key: &str) -> Self {
        Self {
            sensor,
            blackboard_key: blackboard_key.to_string(),
            name: name.to_string(),
        }
    }
}

impl Node for SensorNode {
    fn tick(&mut self, blackboard: &mut BlackBoard) -> NodeResult {
        match self.sensor.read() {
            Ok(data) => {
                blackboard.set(&self.blackboard_key, data);
                NodeResult::Passed
            }
            Err(_) => NodeResult::Failed,
        }
    }
    
    fn reset(&mut self) {}
    
    fn name(&self) -> &str {
        &self.name
    }
}

// Motion planning integration
pub trait MotionPlanner: Send + Sync {
    fn plan_path(&mut self, start: Pose, goal: Pose, obstacles: &[Obstacle]) -> Result<Path, Box<dyn Error>>;
    fn execute_path(&mut self, path: &Path) -> Result<(), Box<dyn Error>>;
    fn is_path_clear(&self, path: &Path) -> bool;
}

pub struct NavigationNode {
    planner: Box<dyn MotionPlanner>,
    current_path: Option<Path>,
    state: NavigationState,
    name: String,
}

#[derive(Debug, Clone)]
enum NavigationState {
    Planning,
    Executing,
    Complete,
    Failed,
}

impl NavigationNode {
    pub fn new(name: &str, planner: Box<dyn MotionPlanner>) -> Self {
        Self {
            planner,
            current_path: None,
            state: NavigationState::Planning,
            name: name.to_string(),
        }
    }
}

impl Node for NavigationNode {
    fn tick(&mut self, blackboard: &mut BlackBoard) -> NodeResult {
        match self.state {
            NavigationState::Planning => {
                if let (Some(start), Some(goal)) = (
                    blackboard.get::<Pose>("current_pose"),
                    blackboard.get::<Pose>("goal_pose")
                ) {
                    let obstacles = blackboard.get::<Vec<Obstacle>>("obstacles").unwrap_or(&vec![]);
                    
                    match self.planner.plan_path(*start, *goal, obstacles) {
                        Ok(path) => {
                            self.current_path = Some(path);
                            self.state = NavigationState::Executing;
                            NodeResult::Running
                        }
                        Err(_) => {
                            self.state = NavigationState::Failed;
                            NodeResult::Failed
                        }
                    }
                } else {
                    NodeResult::Failed
                }
            }
            NavigationState::Executing => {
                if let Some(ref path) = self.current_path {
                    if self.planner.is_path_clear(path) {
                        match self.planner.execute_path(path) {
                            Ok(_) => {
                                self.state = NavigationState::Complete;
                                NodeResult::Passed
                            }
                            Err(_) => {
                                self.state = NavigationState::Failed;
                                NodeResult::Failed
                            }
                        }
                    } else {
                        // Path blocked, need to replan
                        self.state = NavigationState::Planning;
                        NodeResult::Running
                    }
                } else {
                    NodeResult::Failed
                }
            }
            NavigationState::Complete => NodeResult::Passed,
            NavigationState::Failed => NodeResult::Failed,
        }
    }
    
    fn reset(&mut self) {
        self.current_path = None;
        self.state = NavigationState::Planning;
    }
    
    fn name(&self) -> &str {
        &self.name
    }
}
```

## Part 2: Design Patterns & Architecture

### 1. Core Design Patterns Used

#### Strategy Pattern
```rust
// Different execution strategies for composite nodes
pub trait ExecutionStrategy {
    fn execute(&mut self, children: &mut [Box<dyn Node>], blackboard: &mut BlackBoard) -> NodeResult;
}

pub struct SequenceStrategy;
pub struct SelectorStrategy;
pub struct ParallelStrategy { policy: ParallelPolicy }

// Composite node using strategy pattern
pub struct CompositeNode {
    strategy: Box<dyn ExecutionStrategy>,
    children: Vec<Box<dyn Node>>,
    name: String,
}
```

#### Decorator Pattern
```rust
// Base decorator that wraps any node
pub struct DecoratorNode {
    child: Box<dyn Node>,
    decorator: Box<dyn Decorator>,
    name: String,
}

pub trait Decorator {
    fn decorate(&mut self, child_result: NodeResult, blackboard: &mut BlackBoard) -> NodeResult;
    fn should_tick_child(&self) -> bool;
}
```

#### Observer Pattern
```rust
// For debugging and monitoring
pub trait TreeObserver {
    fn on_node_enter(&mut self, node_name: &str);
    fn on_node_exit(&mut self, node_name: &str, result: NodeResult);
    fn on_tree_tick(&mut self);
}

pub struct BehavioralTree {
    root: Box<dyn Node>,
    blackboard: BlackBoard,
    observers: Vec<Box<dyn TreeObserver>>,
}
```

#### Factory Pattern
```rust
// Node factory for creating nodes from configuration
pub trait NodeFactory {
    fn create_node(&self, config: &NodeConfig) -> Result<Box<dyn Node>, Box<dyn Error>>;
}

pub struct DefaultNodeFactory;

impl NodeFactory for DefaultNodeFactory {
    fn create_node(&self, config: &NodeConfig) -> Result<Box<dyn Node>, Box<dyn Error>> {
        match config.node_type.as_str() {
            "sequence" => Ok(Box::new(Sequence::new(&config.name, config.children))),
            "selector" => Ok(Box::new(Selector::new(&config.name, config.children))),
            "action" => Ok(Box::new(Action::new(&config.name, config.action))),
            _ => Err("Unknown node type".into()),
        }
    }
}
```

#### Builder Pattern
```rust
// Tree builder for fluent API
pub struct TreeBuilder {
    factory: Box<dyn NodeFactory>,
}

impl TreeBuilder {
    pub fn new() -> Self {
        Self {
            factory: Box::new(DefaultNodeFactory),
        }
    }
    
    pub fn sequence(name: &str) -> SequenceBuilder {
        SequenceBuilder::new(name)
    }
    
    pub fn selector(name: &str) -> SelectorBuilder {
        SelectorBuilder::new(name)
    }
    
    pub fn action(name: &str, action: fn(&mut BlackBoard) -> NodeResult) -> Box<dyn Node> {
        Box::new(Action::new(name, action))
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
```

### 2. Library Architecture

#### Layered Architecture
```
┌─────────────────────────────────────────────────────────────┐
│                    Application Layer                        │
│  (Robot Controllers, Game AI, Automation Systems)          │
└─────────────────────────────────────────────────────────────┘
┌─────────────────────────────────────────────────────────────┐
│                   High-Level API Layer                     │
│     (TreeBuilder, Configurations, Presets)                 │
└─────────────────────────────────────────────────────────────┘
┌─────────────────────────────────────────────────────────────┐
│                   Behavioral Tree Layer                    │
│   (BehavioralTree, Node Implementations, Execution)        │
└─────────────────────────────────────────────────────────────┘
┌─────────────────────────────────────────────────────────────┐
│                     Core Layer                             │
│      (Node Trait, NodeResult, BlackBoard)                  │
└─────────────────────────────────────────────────────────────┘
┌─────────────────────────────────────────────────────────────┐
│                  Integration Layer                         │
│    (Sensors, Actuators, ROS, External Systems)             │
└─────────────────────────────────────────────────────────────┘
```

#### Module Structure
```
neurotree/
├── src/
│   ├── lib.rs                   # Public API exports
│   ├── core/
│   │   ├── mod.rs
│   │   ├── node.rs              # Node trait, NodeResult
│   │   ├── blackboard.rs        # State management
│   │   └── tree.rs              # BehavioralTree struct
│   ├── nodes/
│   │   ├── mod.rs
│   │   ├── composite/
│   │   │   ├── mod.rs
│   │   │   ├── sequence.rs
│   │   │   ├── selector.rs
│   │   │   └── parallel.rs
│   │   ├── decorator/
│   │   │   ├── mod.rs
│   │   │   ├── inverter.rs
│   │   │   ├── timeout.rs
│   │   │   ├── retry.rs
│   │   │   └── cooldown.rs
│   │   └── leaf/
│   │       ├── mod.rs
│   │       ├── action.rs
│   │       ├── condition.rs
│   │       └── async_action.rs
│   ├── robotics/
│   │   ├── mod.rs
│   │   ├── sensor.rs            # Sensor integration
│   │   ├── navigation.rs        # Motion planning
│   │   └── ros.rs               # ROS integration
│   ├── builder/
│   │   ├── mod.rs
│   │   ├── tree_builder.rs      # Fluent API
│   │   └── factory.rs           # Node factory
│   └── utils/
│       ├── mod.rs
│       ├── observer.rs          # Debugging/monitoring
│       └── serialization.rs     # Tree persistence
└── examples/
    ├── basic_robot.rs
    ├── navigation.rs
    └── sensor_fusion.rs
```

### 3. Implementation Strategy

#### Phase 1: Core Framework
1. **Enhanced Node Trait**
2. **Stateful Composite Nodes**
3. **Basic Decorators**
4. **Improved BlackBoard**

#### Phase 2: Advanced Features
1. **Parallel Execution**
2. **Async Actions**
3. **Builder Pattern API**
4. **Debugging Tools**

#### Phase 3: Robotics Integration
1. **Sensor Framework**
2. **Motion Planning Integration**
3. **ROS Bindings**
4. **Real-time Execution**

#### Phase 4: Production Ready
1. **Serialization/Deserialization**
2. **Performance Optimization**
3. **Visual Editor Support**
4. **Comprehensive Testing**

### 4. Usage Examples

#### Fluent API Example
```rust
let tree = TreeBuilder::new()
    .selector("root")
        .child(
            TreeBuilder::sequence("navigate")
                .child(TreeBuilder::condition("path_clear", |bb| check_path_clear(bb)))
                .child(TreeBuilder::action("move", |bb| move_robot(bb)))
                .build()
        )
        .child(
            TreeBuilder::sequence("avoid_obstacle")
                .child(TreeBuilder::action("scan", |bb| scan_environment(bb)))
                .child(TreeBuilder::action("plan", |bb| plan_avoidance(bb)))
                .child(TreeBuilder::action("execute", |bb| execute_avoidance(bb)))
                .build()
        )
    .build();
```

#### Robotics Integration Example
```rust
// Robot navigation with sensor fusion
let mut robot_tree = TreeBuilder::new()
    .parallel("main_loop", ParallelPolicy::RequireAll)
        .child(
            TreeBuilder::sequence("sensor_fusion")
                .child(SensorNode::new("lidar", lidar_sensor, "lidar_data"))
                .child(SensorNode::new("camera", camera_sensor, "camera_data"))
                .child(TreeBuilder::action("fuse", |bb| fuse_sensor_data(bb)))
                .build()
        )
        .child(
            TreeBuilder::selector("navigation")
                .child(NavigationNode::new("navigate", motion_planner))
                .child(TreeBuilder::action("emergency_stop", |bb| emergency_stop(bb)))
                .build()
        )
    .build();
```

This architecture provides a robust, extensible foundation for robotics applications while maintaining clean separation of concerns and following established design patterns.