# Architecture Changes Needed for Full wgpu Support

## The Core Problem
Current architecture: **Synchronous render loop in event handler**
```rust
WindowEvent::RedrawRequested => {
    graph.execute(&mut backend)?;  // BLOCKS until frame complete
    window.request_redraw();       // Immediate next frame
}
```

This creates rapid fire acquire() calls that exhaust the swapchain.

## Required Architecture Changes

### 1. **Async Render Pipeline**
Instead of synchronous execute(), need async submission:

```rust
// Current (synchronous)
fn execute(&mut self, backend: &mut Backend) -> Result<()>

// Needed (async)
async fn submit_frame(&mut self, backend: &mut Backend) -> FrameHandle
fn poll_completion(&mut self, handle: FrameHandle) -> FrameStatus
```

This allows:
- Submit frame N
- Check if frame N-3 is done
- Only request_redraw() when previous frame completes

### 2. **Frame Scheduler / Pacing System**
New component to manage frame timing:

```rust
struct FrameScheduler {
    in_flight: VecDeque<FrameHandle>,
    max_in_flight: usize,
}

impl FrameScheduler {
    fn can_submit_frame(&self) -> bool {
        self.in_flight.len() < self.max_in_flight
    }
    
    fn wait_for_slot(&mut self) {
        // Poll until oldest frame completes
    }
}
```

### 3. **Event Loop Restructuring**
Move from event-driven to tick-based:

```rust
// Current: Event-driven
RedrawRequested => execute()

// Needed: Tick-based with scheduler
AboutToWait => {
    if scheduler.can_submit_frame() {
        execute_async();
        window.request_redraw();
    } else {
        scheduler.poll(); // Non-blocking check
        // request_redraw later when frame completes
    }
}
```

### 4. **Backend API Changes**
Backends need to expose frame lifecycle:

```rust
trait Backend {
    // Current
    fn execute_graph(&mut self) -> Result<()>;
    
    // Needed
    fn begin_frame(&mut self) -> Result<FrameToken>;
    fn submit_frame(&mut self, token: FrameToken) -> FrameHandle;
    fn poll_frame(&mut self, handle: FrameHandle) -> FrameStatus;
    fn wait_for_frame(&mut self, handle: FrameHandle) -> Result<()>;
}
```

### 5. **Double-Buffered Graph State**
Can't build graph while previous frame is rendering:

```rust
struct RenderGraph {
    current: GraphState,
    pending: GraphState,
    
    fn prepare_next_frame(&mut self);
    fn swap(&mut self);  // When frame completes
}
```

## Size of Change

### Small Changes (Days)
- ❌ None - all require fundamental restructuring

### Medium Changes (Weeks)
- Frame scheduler component
- Async graph execution
- Backend API extensions

### Large Changes (Months)
- ✅ **This is a large change**
- Complete event loop rewrite
- All backends need async support
- Graph needs double-buffering
- Testing matrix explodes (3 backends × async/sync × platforms)

## Alternative: Hybrid Approach

Keep sync API for Vulkan/DX, add wgpu-specific path:

```rust
trait Backend {
    fn execute_graph(&mut self) -> Result<()>; // Sync (Vulkan/DX)
    fn supports_async(&self) -> bool;
    fn execute_async(&mut self) -> Result<FrameHandle>; // wgpu only
}
```

But this creates two completely different code paths to maintain.

## Recommendation

**Don't do it.** Here's why:

1. **Maintenance burden**: 2× the complexity for marginal benefit
2. **wgpu is a portability layer**: For final shipping, you'd compile native backends anyway
3. **Limited wgpu advantage**: Only benefits WebGPU target, which is niche for renderers
4. **Better alternatives exist**: 
   - Use wgpu for prototyping/experimentation
   - Vulkan for Linux
   - DirectX for Windows  
   - Metal for macOS (can add later with same architecture)

## What wgpu IS Good For (Current Architecture)

✅ **Headless rendering** - No swapchain issues
✅ **Single-frame tests** - Perfect for CI
✅ **Compute pipelines** - Different submission model
✅ **Prototyping** - Quick platform bring-up

## Bottom Line

To fully support wgpu for interactive rendering, you'd need to:
- Rewrite the entire application loop (event handling)
- Make RenderGraph async
- Add frame scheduling system
- Duplicate backend API (sync + async)
- Test everything 2× (sync + async paths)

**Estimated effort**: 3-4 weeks of work, 50%+ maintenance increase

**Benefit**: Can use wgpu backend for interactive rendering

**Worth it?** Only if shipping to WebGPU is a core requirement.
# How Production wgpu Apps Handle This

## Pattern 1: Frame Throttling via Events

```rust
struct App {
    window: Window,
    frame_in_progress: bool,
}

impl ApplicationHandler for App {
    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::RedrawRequested => {
                if !self.frame_in_progress {
                    self.frame_in_progress = true;
                    self.render();
                    // Don't request_redraw() here!
                }
            }
            _ => {}
        }
    }
    
    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        // Only request redraw if no frame in progress
        if !self.frame_in_progress {
            self.window.request_redraw();
        }
    }
    
    fn render(&mut self) {
        // ... render ...
        surface_texture.present();
        
        // Mark frame complete AFTER present
        // (But present() doesn't actually block on wgpu!)
        self.frame_in_progress = false;
    }
}
```

**Problem**: `frame_in_progress` flag doesn't actually track GPU completion,
just CPU submission. Still exhausts swapchain.

## Pattern 2: Manual Throttling (Sleep)

```rust
WindowEvent::RedrawRequested => {
    render();
    std::thread::sleep(Duration::from_millis(16)); // ~60 FPS
    window.request_redraw();
}
```

**Problem**: 
- Wastes CPU time sleeping
- Not tied to actual GPU completion
- Inconsistent frame pacing

## Pattern 3: VSync Reliance (What We Have)

```rust
surface_config.present_mode = PresentMode::Fifo; // Should block!
```

**Problem**: Doesn't actually block on AMD+Linux Vulkan backend.

## Pattern 4: WebGPU (Where wgpu Shines)

```rust
// In browser, the browser handles frame pacing
requestAnimationFrame(() => {
    render(); // Can't render faster than browser allows
});
```

**This works!** Browser provides external throttling.

## What Real wgpu Apps Do

### Option A: Accept It
Many wgpu examples just run for a few seconds or handle timeouts:

```rust
match surface.get_current_texture() {
    Err(SurfaceError::Timeout) => return, // Skip frame
    // ...
}
```

### Option B: Lower Frame Rate
Set `desired_maximum_frame_latency: 1` and render at 30 FPS:

```rust
let mut last_frame = Instant::now();
if last_frame.elapsed() > Duration::from_millis(33) {
    window.request_redraw();
    last_frame = Instant::now();
}
```

### Option C: Platform-Specific Backends
Use wgpu's backend selection to force non-Vulkan on problematic platforms:

```rust
backends: wgpu::Backends::DX12 | wgpu::Backends::METAL,
// Avoid Vulkan on AMD
```

### Option D: External Frame Sync
Use a separate thread to monitor GPU:

```rust
// Worker thread
loop {
    device.poll(Maintain::Wait); // Block here, not main thread
    tx.send(FrameComplete);
}

// Main thread
if rx.try_recv().is_ok() {
    window.request_redraw();
}
```

**Problem**: Still can't access per-submission fences in wgpu.

## The Truth

**Most wgpu applications targeting native:**
1. Are prototypes that run briefly
2. Use lower frame rates (30 FPS)
3. Target WebGPU where browser handles sync
4. Switch to native backends (Vulkan/DX) for shipping

**Production games using wgpu:**
- Almost none ship with wgpu backend
- Use wgpu for development/prototyping
- Ship with platform-specific backends

## For Our Renderer

### Current State
✅ Perfect for CI (single frames)
✅ Works for Vulkan backend (unlimited)
✅ Works for DX backend (unlimited)
⚠️ wgpu limited to 4-5 frames

### Options

#### Option 1: Keep As-Is (Recommended)
- wgpu for CI only
- Vulkan/DX for real use
- Document limitation
- **Cost**: None
- **Benefit**: What we have works

#### Option 2: Hack It (Not Recommended)
- Add sleep/throttling
- Lower frame rate to 30 FPS
- Accept 4-5 frame limit for demos
- **Cost**: Ugly workarounds
- **Benefit**: wgpu "works" (poorly)

#### Option 3: Async Rewrite (Not Recommended)
- Complete architecture change
- 3-4 weeks work
- All backends need async
- **Cost**: Massive
- **Benefit**: wgpu works well

#### Option 4: wgpu-Specific Path (Middle Ground)
- Keep sync API for Vulkan/DX
- Add async-only path for wgpu
- **Cost**: Moderate (1-2 weeks)
- **Benefit**: Both models work
- **Downside**: Two render paths forever

## My Recommendation

**Option 1**: Keep as-is.

wgpu serves its purpose (CI testing, prototyping).
For production rendering, Vulkan and DirectX are superior anyway.

If you later need WebGPU, the architecture change makes sense.
But for a native renderer, it's not worth the complexity.
