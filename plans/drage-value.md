# Plan: Implement Draggable Number Value Field (Drag Value Widget)

## Overview
Implement an egui-style drag value widget - a number field that displays a value and allows the user to drag left/right to adjust it, or click to enter text input mode. This is the right-side component that will later be combined with a slider.

## Architecture Analysis

Based on exploration of the codebase:

### Existing Patterns
- **Slider** (`crates/astra-gui-interactive/src/slider.rs`): Uses `DragStart`/`DragMove` events, converts position to value
- **Text Input** (`crates/astra-gui-interactive/src/text_input.rs`): Handles focus, cursor, keyboard input, text editing
- **Event System**: Provides `Click`, `DragStart`, `DragMove`, `DragEnd`, `Focus`, `Blur` events
- **Component Pattern**: 
  - UI function returns `Node` tree (declarative, stateless)
  - Update function processes events and mutates application state
  - Returns `bool` if value changed

### Key Files to Modify/Create
- **New**: `crates/astra-gui-interactive/src/drag_value.rs` - Main implementation
- **Modify**: `crates/astra-gui-interactive/src/lib.rs` - Export new module
- **New**: Example file or extend existing examples to demonstrate usage

## Implementation Plan

### Phase 1: Create Basic Drag Value Component Structure

**File**: `crates/astra-gui-interactive/src/drag_value.rs`

1. **Define `DragValueStyle` struct**
   - Font size, font family
   - Text color (normal, hover, active, disabled)
   - Background color/shape
   - Padding
   - Border/outline styles
   - Cursor appearance when dragging
   - Focused text input style

2. **Implement `drag_value()` function** - Returns Node tree
   ```rust
   pub fn drag_value(
       id: impl Into<String>,
       value: f32,
       focused: bool,
       disabled: bool,
       style: &DragValueStyle,
   ) -> Node
   ```
   
   **Behavior**:
   - If `focused == true`: Return a `text_input()` node showing the value as editable text
   - If `focused == false`: Return a stack with:
     - Text node displaying formatted value (e.g., "42.5")
     - Transparent hitbox node for drag detection (with ID for event targeting)
   - Apply hover/active styles via `base_style`, `hover_style`, `active_style`
   - Show left-right cursor icon on hover (via cursor style in shape)

### Phase 2: Implement Drag Interaction Logic

**File**: `crates/astra-gui-interactive/src/drag_value.rs`

3. **Implement `drag_value_update()` function** - Processes events and updates state
   ```rust
   pub fn drag_value_update(
       id: &str,
       value: &mut f32,
       text_buffer: &mut String,
       cursor_pos: &mut usize,
       selection: &mut Option<(usize, usize)>,
       focused: &mut bool,
       events: &[TargetedEvent],
       input_state: &InputState,
       event_dispatcher: &mut EventDispatcher,
       range: Option<RangeInclusive<f32>>,
       speed: f32,
       step: Option<f32>,
   ) -> bool
   ```

   **Logic**:
   - **When NOT focused** (drag mode):
     - Listen for `Click` events → Set `focused = true`, initialize `text_buffer` from value
     - Listen for `DragMove` events:
       - Extract `delta.x` from drag event
       - Calculate value change: `delta_value = delta.x * speed`
       - Apply speed modifiers:
         - If `Shift` held: multiply by 0.1 (slower, precise)
         - If `Ctrl` held: multiply by 10.0 (faster)
       - Update value: `*value += delta_value`
       - Apply optional step snapping
       - Clamp to range if provided
       - Return `true` if value changed
   
   - **When focused** (text input mode):
     - Delegate to `text_input_update()` to handle typing, cursor, selection
     - Listen for:
       - `Enter` key: Parse `text_buffer` to f32, update `value`, set `focused = false`
       - `Escape` key: Revert `text_buffer` to current value, set `focused = false`
       - `Blur` event: Parse text and update value, set `focused = false`
     - Handle parse errors gracefully (keep previous value if invalid)

4. **Add value formatting helper**
   ```rust
   fn format_value(value: f32, precision: usize) -> String
   ```
   - Format float with configurable decimal places
   - Optionally strip trailing zeros (e.g., "42.0" → "42")

5. **Add text parsing helper**
   ```rust
   fn parse_value(text: &str) -> Option<f32>
   ```
   - Parse string to f32
   - Handle edge cases (empty string, invalid input)

### Phase 3: Integration and Styling

6. **Export from module**
   - Add `pub mod drag_value;` to `crates/astra-gui-interactive/src/lib.rs`
   - Export `drag_value`, `drag_value_update`, `DragValueStyle`

7. **Create default style**
   - Implement `Default` for `DragValueStyle`
   - Use sensible defaults matching existing slider/text input styles
   - Consider adding `impl DragValueStyle` with builder methods (e.g., `with_precision()`)

### Phase 4: Example Implementation

**Option A**: Extend existing example (`crates/astra-gui-wgpu/examples/interactive.rs`)
**Option B**: Create new example (`crates/astra-gui-wgpu/examples/drag_value.rs`)

8. **Create example demonstrating:**
   - Basic drag value usage
   - Drag to adjust value (show visual feedback)
   - Click to enter text input mode
   - Speed modifiers (Shift/Ctrl)
   - Optional: Range clamping
   - Optional: Step snapping
   - Multiple drag values with different configurations

   **App State**:
   ```rust
   struct DragValueExample {
       value: f32,
       text_buffer: String,
       cursor_pos: usize,
       selection: Option<(usize, usize)>,
       focused: bool,
       // ... other state
   }
   ```

## Design Decisions

### State Management
- Application stores: `value` (f32), `text_buffer` (String), `cursor_pos`, `selection`, `focused` (bool)
- Component is stateless - state passed in as parameters
- Follows exact pattern from `slider` and `text_input`

### Drag Speed Calculation
- Base speed multiplier (e.g., 0.1 = 1px drag = 0.1 value change)
- Shift modifier: 0.1x (precise)
- Ctrl modifier: 10x (fast)
- Configurable via parameter

### Text Input Integration
- Reuse existing `text_input()` and `text_input_update()` when focused
- No need to reimplement cursor/selection logic
- Seamless transition between drag and text modes

### Visual Feedback
- Hover: Show left-right cursor icon, change background/text color
- Active (dragging): Change to active style
- Focused (text input): Show text cursor, selection highlight
- Disabled: Grayed out, no interaction

## Testing Approach
1. Basic value display
2. Drag left decreases value, drag right increases value
3. Speed modifiers work correctly
4. Click transitions to text input mode
5. Text input parses correctly
6. Invalid text doesn't crash, reverts to previous value
7. Range clamping works
8. Step snapping works

## Future Enhancements (Out of Scope)
- Combine with slider for egui-style slider+drag_value combo widget
- Vertical drag support
- Custom formatters (e.g., units, scientific notation)
- Mouse wheel support for value adjustment
- Double-click for special behavior

## Critical Files
- `crates/astra-gui-interactive/src/drag_value.rs` (new)
- `crates/astra-gui-interactive/src/lib.rs` (modify)
- `crates/astra-gui-interactive/src/slider.rs` (reference)
- `crates/astra-gui-interactive/src/text_input.rs` (reference)
- Example file (new or modify existing)
