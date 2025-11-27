// engine-core/src/compute/wave.rs
//! 2D wave equation finite-difference solver (second-order in time).
//!
//! Discretization (standard explicit scheme):
//! u_{t+dt} = 2 u_t - u_{t-dt} + (c*dt/dx)^2 * Laplacian(u_t)
//!
//! This struct stores three fields: prev (u_{t-dt}), curr (u_t), next (u_{t+dt}).

#[derive(Debug, Clone)]
pub struct Wavefield {
    pub width: usize,
    pub height: usize,
    pub prev: Vec<f32>,
    pub curr: Vec<f32>,
    pub next: Vec<f32>,
}

impl Wavefield {
    /// Create a wavefield initialized to zeros.
    pub fn new(width: usize, height: usize) -> Self {
        let len = width.checked_mul(height).expect("dimensions overflow");
        Self {
            width,
            height,
            prev: vec![0.0; len],
            curr: vec![0.0; len],
            next: vec![0.0; len],
        }
    }

    /// Returns number of elements (width * height).
    pub fn len(&self) -> usize {
        self.width * self.height
    }

    /// Reset all fields to zero.
    pub fn reset(&mut self) {
        for v in &mut self.prev { *v = 0.0; }
        for v in &mut self.curr { *v = 0.0; }
        for v in &mut self.next { *v = 0.0; }
    }

    /// Seed a point impulse at (x, y) into `curr` (optionally with value).
    /// Uses clamping for coordinates.
    pub fn seed_point(&mut self, x: usize, y: usize, value: f32) {
        if x < self.width && y < self.height {
            let idx = y * self.width + x;
            self.curr[idx] = value;
        }
    }

    /// Step the simulation forward by one timestep.
    ///
    /// - `c`: wave speed (units consistent with dt/dx)
    /// - `dt`: timestep
    /// - `dx`: spatial grid spacing (same units as dt*c)
    ///
    /// Boundary conditions: simple zero-Neumann (copy neighbor) implemented by ignoring
    /// the Laplacian at boundaries (i.e., we don't read outside array; edges are left computed with available neighbors).
    pub fn step(&mut self, c: f32, dt: f32, dx: f32) {
        let nx = self.width;
        let ny = self.height;
        let len = self.len();
        if len == 0 { return; }

        // stability factor
        let coeff = (c * dt / dx).powi(2);

        // index helper
        let idx = |x: isize, y: isize| -> Option<usize> {
            if x < 0 || y < 0 { return None; }
            let (xu, yu) = (x as usize, y as usize);
            if xu >= nx || yu >= ny { return None; }
            Some(yu * nx + xu)
        };

        // For each interior cell compute Laplacian using 4 neighbors (5-point stencil)
        for y in 0..ny as isize {
            for x in 0..nx as isize {
                let center_i = idx(x, y).unwrap();
                // fetch neighbor values (if out-of-bounds, use center value => zero Neumann-ish)
                let center = self.curr[center_i];
                let left   = idx(x - 1, y).map(|i| self.curr[i]).unwrap_or(center);
                let right  = idx(x + 1, y).map(|i| self.curr[i]).unwrap_or(center);
                let up     = idx(x, y - 1).map(|i| self.curr[i]).unwrap_or(center);
                let down   = idx(x, y + 1).map(|i| self.curr[i]).unwrap_or(center);

                let lap = left + right + up + down - 4.0 * center;

                // wave update: next = 2*curr - prev + coeff * lap
                self.next[center_i] = 2.0 * center - self.prev[center_i] + coeff * lap;
            }
        }

        // rotate buffers: prev <- curr, curr <- next, next <- prev (reuse vec allocations)
        std::mem::swap(&mut self.prev, &mut self.curr);
        std::mem::swap(&mut self.curr, &mut self.next);

        // optional: clear next (not strictly necessary but keeps expectations)
        for v in &mut self.next { *v = 0.0; }
    }
}
