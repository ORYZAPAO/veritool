use std::collections::HashMap;

use crate::design::{Range, Signal};
use crate::params::{evaluate_expr, ParamEnv};

/// Compute packed × unpacked bit-width of a signal given a parameter environment.
pub fn calculate_width_with_params(signal: &Signal, env: &ParamEnv) -> i64 {
    let packed = range_width(signal.packed_width.as_ref(), env.as_map());
    let unpacked: i64 = if signal.unpacked_dims.is_empty() {
        1
    } else {
        signal
            .unpacked_dims
            .iter()
            .map(|d| range_width(Some(d), env.as_map()))
            .product()
    };
    packed * unpacked
}

/// Same but accepts a raw HashMap for convenience (e.g. from the unit tests).
pub fn calculate_width_with_map(signal: &Signal, params: &HashMap<String, i64>) -> i64 {
    let packed = range_width(signal.packed_width.as_ref(), params);
    let unpacked: i64 = if signal.unpacked_dims.is_empty() {
        1
    } else {
        signal
            .unpacked_dims
            .iter()
            .map(|d| range_width(Some(d), params))
            .product()
    };
    packed * unpacked
}

/// Legacy no-params version (falls back to 1 for unresolvable expressions).
pub fn calculate_width(signal: &Signal) -> i64 {
    calculate_width_with_map(signal, &HashMap::new())
}

/// Same as `calculate_width` but named for FF contexts.
pub fn calculate_ff_width(signal: &Signal) -> i64 {
    calculate_width(signal)
}

// ─── internal ─────────────────────────────────────────────────────────────────

fn range_width(range: Option<&Range>, params: &HashMap<String, i64>) -> i64 {
    match range {
        None => 1,
        Some(r) => {
            let msb = evaluate_expr(&r.msb, params);
            let lsb = evaluate_expr(&r.lsb, params);
            match (msb, lsb) {
                (Some(m), Some(l)) => (m - l).abs() + 1,
                _ => {
                    // Partially evaluable: fall back to 1
                    1
                }
            }
        }
    }
}
