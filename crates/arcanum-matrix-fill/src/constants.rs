use std::f64::consts::PI;

/// Speed of light in vacuum (m/s), exact by SI definition.
pub const C_LIGHT: f64 = 299_792_458.0;

/// Permeability of free space (H/m), exact by SI definition.
pub const MU_0: f64 = 4.0e-7 * PI;

/// Permittivity of free space (F/m), derived from c = 1/√(μ₀ε₀).
pub const EPS_0: f64 = 1.0 / (MU_0 * C_LIGHT * C_LIGHT);

/// Free-space wavenumber k = 2πf/c (rad/m).
pub fn wavenumber(frequency_hz: f64) -> f64 {
    2.0 * PI * frequency_hz / C_LIGHT
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mu0_eps0_c_squared_equals_one() {
        let product = MU_0 * EPS_0 * C_LIGHT * C_LIGHT;
        assert!((product - 1.0).abs() < 1e-15);
    }

    #[test]
    fn wavenumber_at_300_mhz() {
        // At 300 MHz, λ = c/f ≈ 0.9993 m, so k = 2πf/c ≈ 2π × 1.00069.
        let k = wavenumber(300e6);
        let expected = 2.0 * PI * 300e6 / C_LIGHT;
        assert!((k - expected).abs() < 1e-15);
    }

    #[test]
    fn wavenumber_zero_frequency() {
        assert_eq!(wavenumber(0.0), 0.0);
    }
}
