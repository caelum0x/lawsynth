use lawsynth_plugin_api::{AlgorithmPlugin, AlgorithmRequest, AlgorithmResponse, DataBatch, PluginError};

#[derive(Clone, Debug)]
pub struct LinearOperator {
    pub minimum_variance: f64,
}

impl Default for LinearOperator {
    fn default() -> Self { Self { minimum_variance: 1.0e-12 } }
}

fn numeric(batch: &DataBatch) -> Result<Vec<Option<f64>>, PluginError> {
    match batch {
        DataBatch::Float64(v) => Ok(v.iter().copied().map(Some).collect()),
        DataBatch::Int64(v) => Ok(v.iter().map(|&x| Some(x as f64)).collect()),
        DataBatch::NullableFloat64(v) => Ok(v.clone()),
        DataBatch::NullableInt64(v) => Ok(v.iter().map(|x| x.map(|n| n as f64)).collect()),
        _ => Err(PluginError::Unsupported("linear operator requires numeric columns".into())),
    }
}

impl AlgorithmPlugin for LinearOperator {
    fn discover(&self, request: AlgorithmRequest) -> Result<AlgorithmResponse, PluginError> {
        request.validate()?;
        if !self.minimum_variance.is_finite() || self.minimum_variance <= 0.0 {
            return Err(PluginError::InvalidData("minimum_variance must be finite and positive".into()));
        }
        let target_index = request.schema.columns.iter().position(|c| c.name == request.target)
            .ok_or_else(|| PluginError::InvalidData("target column is absent".into()))?;
        let target = numeric(&request.columns[target_index])?;
        let mut best: Option<(&str, f64, f64)> = None;
        for (index, column) in request.schema.columns.iter().enumerate() {
            if index == target_index { continue; }
            let values = match numeric(&request.columns[index]) { Ok(v) => v, Err(_) => continue };
            let pairs: Vec<(f64, f64)> = values.into_iter().zip(target.iter().copied())
                .filter_map(|(x, y)| x.zip(y)).collect();
            if pairs.len() < 2 { continue; }
            let mean_x = pairs.iter().map(|p| p.0).sum::<f64>() / pairs.len() as f64;
            let mean_y = pairs.iter().map(|p| p.1).sum::<f64>() / pairs.len() as f64;
            let variance = pairs.iter().map(|p| (p.0 - mean_x).powi(2)).sum::<f64>();
            if variance < self.minimum_variance { continue; }
            let slope = pairs.iter().map(|p| (p.0 - mean_x) * (p.1 - mean_y)).sum::<f64>() / variance;
            let intercept = mean_y - slope * mean_x;
            let error = pairs.iter().map(|p| (p.1 - (intercept + slope * p.0)).powi(2)).sum::<f64>() / pairs.len() as f64;
            if best.is_none_or(|(_, _, current)| error < current) { best = Some((&column.name, slope, error)); }
        }
        let (feature, slope, error) = best.ok_or_else(|| PluginError::Unsupported("no usable numeric predictor".into()))?;
        let response = AlgorithmResponse {
            equation: format!("d({})/dt = {:.17} * {}", request.target, slope, feature),
            score: -error,
            diagnostics: vec![format!("mean_squared_error={error:.17}")],
        };
        response.validate()?;
        Ok(response)
    }
}
