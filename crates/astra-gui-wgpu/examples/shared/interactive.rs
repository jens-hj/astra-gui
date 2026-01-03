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

    /// Call at START of frame for transition timing
    pub fn begin_frame_transitions(&mut self) {
        self.state_manager.begin_frame();
    }

    /// Call at END of frame to clear input state
    pub fn end_frame(&mut self) {
        self.input_state.begin_frame();
    }

    /// DEPRECATED: Do not use - split into begin_frame_transitions() and end_frame()
    #[deprecated(note = "Use begin_frame_transitions() at start and end_frame() at end")]
    pub fn begin_frame(&mut self) {
        self.begin_frame_transitions();
        self.end_frame();
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
