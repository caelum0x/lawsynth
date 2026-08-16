use crate::{IntervalConfig, UncertaintyError};

/// A single constrained-profile observation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ProfilePoint {
    pub parameter: f64,
    pub objective: f64,
}

/// A profile likelihood summary fitted to a local quadratic objective.
#[derive(Clone, Debug, PartialEq)]
pub struct ProfileResult {
    pub optimum: f64,
    pub minimum: f64,
    pub curvature: f64,
    pub interval: (f64, f64),
}

/// Fits `objective = a*x² + b*x + c` and returns the unit-increase profile interval.
pub fn profile_quadratic(
    points: &[ProfilePoint],
    config: IntervalConfig,
) -> Result<ProfileResult, UncertaintyError> {
    config.validate()?;
    if points.len() < 3 {
        return Err(UncertaintyError::TooFewSamples {
            minimum: 3,
            actual: points.len(),
        });
    }
    if points
        .iter()
        .any(|p| !p.parameter.is_finite() || !p.objective.is_finite())
    {
        return Err(UncertaintyError::NonFiniteValue);
    }
    let mut normal = [[0.0; 3]; 3];
    let mut rhs = [0.0; 3];
    for point in points {
        let basis = [point.parameter * point.parameter, point.parameter, 1.0];
        for i in 0..3 {
            rhs[i] += basis[i] * point.objective;
            for j in 0..3 {
                normal[i][j] += basis[i] * basis[j];
            }
        }
    }
    let coefficients = solve_3(normal, rhs)?;
    let curvature = coefficients[0];
    if curvature <= 0.0 {
        return Err(UncertaintyError::NonPositiveVariance);
    }
    let optimum = -coefficients[1] / (2.0 * curvature);
    let minimum = coefficients[2] - coefficients[1] * coefficients[1] / (4.0 * curvature);
    // Normal approximation to a two-sided interval: the z multiplier is
    // calculated with a rational inverse-normal approximation.
    let z = inverse_normal(0.5 + config.confidence / 2.0);
    let radius = z / curvature.sqrt();
    Ok(ProfileResult {
        optimum,
        minimum,
        curvature,
        interval: (optimum - radius, optimum + radius),
    })
}

fn solve_3(mut matrix: [[f64; 3]; 3], mut rhs: [f64; 3]) -> Result<[f64; 3], UncertaintyError> {
    for pivot in 0..3 {
        let row = (pivot..3)
            .max_by(|&a, &b| matrix[a][pivot].abs().total_cmp(&matrix[b][pivot].abs()))
            .unwrap();
        if matrix[row][pivot].abs() < 1e-14 {
            return Err(UncertaintyError::SingularCovariance);
        }
        matrix.swap(pivot, row);
        rhs.swap(pivot, row);
        let scale = matrix[pivot][pivot];
        for value in &mut matrix[pivot][pivot..] {
            *value /= scale;
        }
        rhs[pivot] /= scale;
        let normalized_pivot = matrix[pivot];
        let normalized_rhs = rhs[pivot];
        for other in 0..3 {
            if other != pivot {
                let factor = matrix[other][pivot];
                for (target, source) in matrix[other][pivot..]
                    .iter_mut()
                    .zip(normalized_pivot[pivot..].iter())
                {
                    *target -= factor * source;
                }
                rhs[other] -= factor * normalized_rhs;
            }
        }
    }
    Ok(rhs)
}

fn inverse_normal(p: f64) -> f64 {
    // Peter J. Acklam's rational approximation, sufficient for interval scaling.
    let a = [
        -39.696_830_286_653_8,
        220.946_098_424_521,
        -275.928_510_446_969,
        138.357_751_867_269,
        -30.664_798_066_147_2,
        2.506_628_277_459_24,
    ];
    let b = [
        -54.476_098_798_224_1,
        161.585_836_858_041,
        -155.698_979_859_887,
        66.801_311_887_719_7,
        -13.280_681_552_885_7,
    ];
    let c = [
        -0.007_784_894_002_430_29,
        -0.322_396_458_041_136,
        -2.400_758_277_161_84,
        -2.549_732_539_343_73,
        4.374_664_141_464_97,
        2.938_163_982_698_78,
    ];
    let d = [
        0.007_784_695_709_041_46,
        0.322_467_129_070_04,
        2.445_134_137_143,
        3.754_408_661_907_42,
    ];
    if p < 0.02425 {
        let q = (-2.0 * p.ln()).sqrt();
        return (((((c[0] * q + c[1]) * q + c[2]) * q + c[3]) * q + c[4]) * q + c[5])
            / ((((d[0] * q + d[1]) * q + d[2]) * q + d[3]) * q + 1.0);
    }
    if p > 0.97575 {
        return -inverse_normal(1.0 - p);
    }
    let q = p - 0.5;
    let r = q * q;
    (((((a[0] * r + a[1]) * r + a[2]) * r + a[3]) * r + a[4]) * r + a[5]) * q
        / (((((b[0] * r + b[1]) * r + b[2]) * r + b[3]) * r + b[4]) * r + 1.0)
}
