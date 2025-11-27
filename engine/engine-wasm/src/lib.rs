use wasm_bindgen::prelude::*;
use engine_core::compute;
use engine_core::compute::wave::Wavefield;

#[wasm_bindgen(start)]
pub fn start() {
    #[cfg(debug_assertions)]
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
pub fn init() -> Result<(), JsValue> {
    #[cfg(debug_assertions)]
    console_error_panic_hook::set_once();
    Ok(())
}

//
// ─────────────────────────────────────────────────────────────
//   BASIC EXPORTS (unchanged)
// ─────────────────────────────────────────────────────────────
//

#[wasm_bindgen]
pub fn wasm_sieve(n: usize) -> Vec<usize> {
    compute::sieve::sieve(n)
}

/// Legacy: in-place wavefield update for 1D/flat arrays
#[wasm_bindgen]
pub fn wasm_step_wavefield(ptr: *mut f32, len: usize, c: f32, dt: f32, dx: f32) {
    let slice = unsafe { std::slice::from_raw_parts_mut(ptr, len) };
    compute::wave::step_wavefield(slice, c, dt, dx);
}

//
// ─────────────────────────────────────────────────────────────
//   2D WAVEFIELD SIMULATOR (full engine)
// ─────────────────────────────────────────────────────────────
//

#[wasm_bindgen]
pub struct WasmWavefield {
    inner: Wavefield,
}

#[wasm_bindgen]
impl WasmWavefield {
    /// Create a new 2D wavefield
    #[wasm_bindgen(constructor)]
    pub fn new(width: usize, height: usize) -> WasmWavefield {
        WasmWavefield {
            inner: Wavefield::new(width, height),
        }
    }

    #[wasm_bindgen(getter)]
    pub fn width(&self) -> usize {
        self.inner.width
    }

    #[wasm_bindgen(getter)]
    pub fn height(&self) -> usize {
        self.inner.height
    }

    /// Pointer to current wavefield buffer (u_t)
    #[wasm_bindgen]
    pub fn ptr_curr(&self) -> *const f32 {
        self.inner.curr.as_ptr()
    }

    /// Pointer to previous buffer (u_{t-dt})
    #[wasm_bindgen]
    pub fn ptr_prev(&self) -> *const f32 {
        self.inner.prev.as_ptr()
    }

    /// Pointer to next buffer (u_{t+dt})
    #[wasm_bindgen]
    pub fn ptr_next(&self) -> *const f32 {
        self.inner.next.as_ptr()
    }

    /// Advance simulation one timestep
    #[wasm_bindgen]
    pub fn step(&mut self, c: f32, dt: f32, dx: f32) {
        self.inner.step(c, dt, dx);
    }

    /// Seed a single point in current field
    #[wasm_bindgen]
    pub fn seed_point(&mut self, x: usize, y: usize, value: f32) {
        self.inner.seed_point(x, y, value);
    }

    /// Zero all three buffers
    #[wasm_bindgen]
    pub fn reset(&mut self) {
        self.inner.reset();
    }
}

