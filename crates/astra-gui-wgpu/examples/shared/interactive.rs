use astra_gui_wgpu::{EventDispatcher, InputState, InteractiveStateManager};

/// Bundle of interactive component state
/// Examples that need interactivity just store this struct
pub struct InteractiveState {
    pub input_state: InputState,
    pub event_dispatcher: EventDispatcher,
    pub state_manager: InteractiveStateManager,
    pub last_frame_time: std::time::Instant,
}

impl InteractiveState {
    pub fn new() -> Self {
        Self {
            input_state: InputState::new(),
            event_dispatcher: EventDispatcher::new(),
            state_manager: InteractiveStateManager::new(),
            last_frame_time: std::time::Instant::now(),
        }
    }

    /// Call at the beginning of each frame
    pub fn begin_frame(&mut self) {
        self.state_manager.begin_frame();
        self.input_state.begin_frame();
    }

    /// Calculate delta time since last frame
    pub fn delta_time(&mut self) -> std::time::Duration {
        let now = std::time::Instant::now();
        let delta = now.duration_since(self.last_frame_time);
        self.last_frame_time = now;
        delta
    }
}

impl Default for InteractiveState {
    fn default() -> Self {
        Self::new()
    }
}
