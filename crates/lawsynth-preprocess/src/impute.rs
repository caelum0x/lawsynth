use crate::PreprocessError;

/// Explicit strategy used to fill missing pre-ingestion observations.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ImputationMethod {
    ForwardFill,
    Mean,
    Linear,
}

/// Immutable metadata describing which source positions were filled.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ImputationReport {
    pub method: ImputationMethod,
    pub imputed_indices: Vec<usize>,
}

/// Fills nullable numeric observations before they are admitted to the finite
/// Dataset boundary. Linear interpolation intentionally refuses leading or
/// trailing gaps because there is no bracketing evidence for those values.
pub fn impute_series(
    time: &[f64],
    values: &[Option<f64>],
    method: ImputationMethod,
) -> Result<(Vec<f64>, ImputationReport), PreprocessError> {
    if time.len() != values.len() {
        return Err(PreprocessError::ImputationLengthMismatch);
    }
    if time.iter().any(|value| !value.is_finite())
        || values.iter().flatten().any(|value| !value.is_finite())
    {
        return Err(PreprocessError::NonFiniteImputationValue);
    }
    if time.windows(2).any(|pair| pair[1] <= pair[0]) {
        return Err(PreprocessError::InvalidTargetTime);
    }
    let imputed_indices = values
        .iter()
        .enumerate()
        .filter_map(|(index, value)| value.is_none().then_some(index))
        .collect::<Vec<_>>();
    if imputed_indices.is_empty() {
        return Ok((
            values.iter().map(|value| value.expect("checked complete")).collect(),
            ImputationReport { method, imputed_indices },
        ));
    }
    let observed = values.iter().flatten().copied().collect::<Vec<_>>();
    if observed.is_empty() {
        return Err(PreprocessError::NoObservedValues);
    }
    let output = match method {
        ImputationMethod::Mean => {
            let mean = observed.iter().sum::<f64>() / observed.len() as f64;
            values.iter().map(|value| value.unwrap_or(mean)).collect()
        }
        ImputationMethod::ForwardFill => forward_fill(values)?,
        ImputationMethod::Linear => linear_interpolate(time, values)?,
    };
    Ok((output, ImputationReport { method, imputed_indices }))
}

fn forward_fill(values: &[Option<f64>]) -> Result<Vec<f64>, PreprocessError> {
    let mut output = Vec::with_capacity(values.len());
    let mut previous = None;
    for value in values {
        match value {
            Some(value) => {
                previous = Some(*value);
                output.push(*value);
            }
            None => output.push(previous.ok_or(PreprocessError::MissingBoundaryValue)?),
        }
    }
    Ok(output)
}

fn linear_interpolate(time: &[f64], values: &[Option<f64>]) -> Result<Vec<f64>, PreprocessError> {
    if values.first().is_none_or(Option::is_none) || values.last().is_none_or(Option::is_none) {
        return Err(PreprocessError::MissingBoundaryValue);
    }
    let mut output = values.iter().map(|value| value.unwrap_or(0.0)).collect::<Vec<_>>();
    let mut index = 0;
    while index < values.len() {
        if values[index].is_some() {
            index += 1;
            continue;
        }
        let start = index - 1;
        while index < values.len() && values[index].is_none() {
            index += 1;
        }
        let end = index;
        let start_value = output[start];
        let end_value = output[end];
        let duration = time[end] - time[start];
        if duration <= 0.0 {
            return Err(PreprocessError::InvalidTargetTime);
        }
        for position in start + 1..end {
            let fraction = (time[position] - time[start]) / duration;
            output[position] = start_value + fraction * (end_value - start_value);
        }
    }
    Ok(output)
}
