use faer::complex_native::c64;
use faer::Mat;

/// Dense N×N complex impedance matrix.
///
/// Wraps [`faer::Mat<c64>`] to provide the interface needed by the matrix fill
/// pipeline, including an unsafe concurrent-write method for Rayon parallelism.
///
/// # Thread-Safety Contract for Parallel Writes
///
/// The [`write`](ZMatrix::write) method uses raw pointer access to write a
/// single matrix element without requiring `&mut self`. This is sound **only**
/// when all of the following invariants hold:
///
/// 1. The `ZMatrix` is fully allocated before the parallel region begins and
///    is not resized or dropped during it.
/// 2. Each `(row, col)` pair is written by exactly one Rayon thread — no two
///    threads write to the same cell.
/// 3. No concurrent reads occur during the parallel fill phase.
/// 4. `c64` is `Copy + Send` — each write is a plain 16-byte store.
///
/// These invariants are upheld by the upper-triangle element classification in
/// [`classify`](crate::classify) and the fill orchestration in
/// [`fill_impedance_matrix`](crate::fill_impedance_matrix).
pub struct ZMatrix {
    data: Mat<c64>,
    n_segments: usize,
}

// SAFETY: ZMatrix is safe to send across threads. The inner `Mat<c64>` owns
// its allocation and `c64` is `Send`. Concurrent access is governed by the
// safety contract documented above — callers must ensure non-overlapping writes.
unsafe impl Send for ZMatrix {}
unsafe impl Sync for ZMatrix {}

impl ZMatrix {
    /// Create a zero-initialized N×N impedance matrix.
    pub fn new(n: usize) -> Self {
        Self {
            data: Mat::zeros(n, n),
            n_segments: n,
        }
    }

    /// Read element Z\[row, col\] (bounds-checked).
    pub fn read(&self, row: usize, col: usize) -> c64 {
        self.data.read(row, col)
    }

    /// Write element Z\[row, col\] via raw pointer access.
    ///
    /// # Safety
    ///
    /// See the [thread-safety contract](ZMatrix#thread-safety-contract-for-parallel-writes)
    /// on the struct. The caller must guarantee that no other thread reads or
    /// writes the same `(row, col)` cell concurrently.
    pub unsafe fn write(&self, row: usize, col: usize, val: c64) {
        assert!(row < self.n_segments && col < self.n_segments);
        let ptr = self.data.as_ptr() as *mut c64;
        let col_stride = self.data.col_stride();
        // faer uses column-major layout; row stride is 1 for owned Mat.
        let offset = col as isize * col_stride + row as isize;
        *ptr.offset(offset) = val;
    }

    /// Number of segments (matrix dimension).
    pub fn n_segments(&self) -> usize {
        self.n_segments
    }

    /// Borrow the underlying `faer::Mat<c64>`.
    pub fn data(&self) -> &Mat<c64> {
        &self.data
    }

    /// Consume the wrapper and return the inner `faer::Mat<c64>`.
    pub fn into_inner(self) -> Mat<c64> {
        self.data
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_creates_zero_matrix() {
        let z = ZMatrix::new(4);
        assert_eq!(z.n_segments(), 4);
        for r in 0..4 {
            for c in 0..4 {
                let v = z.read(r, c);
                assert_eq!(v.re, 0.0);
                assert_eq!(v.im, 0.0);
            }
        }
    }

    #[test]
    fn write_and_read_single_element() {
        let z = ZMatrix::new(3);
        let val = c64::new(1.5, -2.3);
        unsafe { z.write(1, 2, val) };
        let got = z.read(1, 2);
        assert_eq!(got.re, val.re);
        assert_eq!(got.im, val.im);
        // Other elements remain zero.
        assert_eq!(z.read(0, 0).re, 0.0);
    }

    #[test]
    fn write_all_elements() {
        let n = 5;
        let z = ZMatrix::new(n);
        for r in 0..n {
            for c in 0..n {
                let val = c64::new(r as f64, c as f64);
                unsafe { z.write(r, c, val) };
            }
        }
        for r in 0..n {
            for c in 0..n {
                let v = z.read(r, c);
                assert_eq!(v.re, r as f64);
                assert_eq!(v.im, c as f64);
            }
        }
    }

    #[test]
    fn into_inner_returns_mat() {
        let z = ZMatrix::new(2);
        unsafe { z.write(0, 1, c64::new(3.0, 4.0)) };
        let mat = z.into_inner();
        assert_eq!(mat.nrows(), 2);
        assert_eq!(mat.ncols(), 2);
        assert_eq!(mat.read(0, 1).re, 3.0);
    }

    #[test]
    #[should_panic]
    fn write_out_of_bounds_panics() {
        let z = ZMatrix::new(2);
        unsafe { z.write(2, 0, c64::new(0.0, 0.0)) };
    }
}
