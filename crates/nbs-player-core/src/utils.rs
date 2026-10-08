use crate::types::Rgba;

/// Formats a time in seconds to a string in the format "mm:ss".
#[inline]
pub fn time_formatter(time: f32) -> String {
    let minutes = (time / 60.0).floor() as u32;
    let seconds = (time % 60.0) as u32;
    format!("{:0>2}:{:0>2}", minutes, seconds)
}

/// Linear interpolation function for smooth animation
#[inline]
pub fn lerp(start: f32, end: f32, t: f32) -> f32 {
    start.mul_add(1.0 - t, end * t)
}

/// Fast approximation for 2^x
pub fn fast_pow2(x: f32) -> f32 {
    let x0 = x.floor();
    let x1 = x - x0;

    // Handle overflow and underflow
    if x0 >= 32.0 {
        return f32::INFINITY; // 2^x is too large for f32
    } else if x0 <= -32.0 {
        return 0.0; // 2^x is too small for f32
    }

    // Calculate 2^x1 using a polynomial approximation
    let p = 1.0 + x1 * (0.693147 + x1 * (0.241586 + x1 * 0.052043));

    // Calculate 2^x0 using bit shifting (only for positive x0)
    if x0 >= 0.0 {
        p * (1 << x0 as i32) as f32
    } else {
        p / (1 << (-x0 as i32)) as f32
    }
}

pub fn blend_colors(base: Rgba, tints: &[(Rgba, f32)]) -> Rgba {
    let mut r = base.r as f32;
    let mut g = base.g as f32;
    let mut b = base.b as f32;
    let mut a = base.a as f32;

    let n = tints.len().max(1) as f32;
    let inv_n = 1.0 / n;

    for (i, (color, weight)) in tints.iter().enumerate() {
        let position_factor = (n - i as f32) * inv_n;
        let effective_weight = weight * position_factor;
        let inv_weight = 1.0 - effective_weight;

        r = r * inv_weight + color.r as f32 * effective_weight;
        g = g * inv_weight + color.g as f32 * effective_weight;
        b = b * inv_weight + color.b as f32 * effective_weight;
        a = a * inv_weight + color.a as f32 * effective_weight;
    }

    Rgba::new(
        r.round() as u8,
        g.round() as u8,
        b.round() as u8,
        a.round() as u8,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pow2_within_a_quarter_percent() {
        for i in -24..=24 {
            let x = i as f32 / 4.0;
            let exact = 2f32.powf(x);
            assert!((fast_pow2(x) - exact).abs() / exact < 5e-3, "x={x}");
        }
    }

    #[test]
    fn time_format() {
        assert_eq!(time_formatter(125.0), "02:05");
    }
}
