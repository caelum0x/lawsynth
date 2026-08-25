//! Bounded information-cascade calibration and intervention forecasting.
//!
//! Edges are observational candidate paths, never causal proof. The app fits a
//! global independent-cascade probability from explicit observation windows,
//! measures chronological holdout quality, and produces seeded process bands.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::{Display, Formatter};
use std::time::{Duration, Instant};

use lawsynth_bundle::sha256_hex;

pub const SCHEMA_VERSION: &str = "lawsynth.information-diffusion.v1";
pub const MODEL_VERSION: &str = "lawsynth-independent-cascade-v1";
const MAX_RUNTIME: Duration = Duration::from_secs(600);

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Edge {
    pub source: String,
    pub target: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Activation {
    pub node: String,
    pub step: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Cascade {
    pub cascade_id: String,
    /// Canonical UTC RFC3339 (`YYYY-MM-DDTHH:MM:SSZ`) for deterministic sorting.
    pub started_at: String,
    pub observation_end_step: usize,
    pub activations: Vec<Activation>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Input {
    pub nodes: Vec<String>,
    pub edges: Vec<Edge>,
    pub cascades: Vec<Cascade>,
    pub seeds: Vec<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Config {
    pub horizon: usize,
    pub simulations: usize,
    pub random_seed: u64,
    pub blocked_nodes: Vec<String>,
    pub transmission_multiplier: f64,
    pub max_runtime: Duration,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            horizon: 30,
            simulations: 1_000,
            random_seed: 42,
            blocked_nodes: Vec::new(),
            transmission_multiplier: 0.75,
            max_runtime: Duration::from_secs(30),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Limits {
    pub max_nodes: usize,
    pub max_edges: usize,
    pub max_cascades: usize,
    pub max_activations: usize,
    pub max_observation_step: usize,
    pub max_calibration_observations: usize,
    pub max_horizon: usize,
    pub max_simulations: usize,
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            max_nodes: 5_000,
            max_edges: 50_000,
            max_cascades: 2_000,
            max_activations: 500_000,
            max_observation_step: 10_000,
            max_calibration_observations: 2_000_000,
            max_horizon: 180,
            max_simulations: 5_000,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Calibration {
    pub probability: f64,
    pub confidence_low: f64,
    pub confidence_high: f64,
    pub observations: usize,
    pub positives: usize,
    pub negative_log_likelihood: f64,
    pub brier_score: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Backtest {
    pub status: &'static str,
    pub train_cascades: usize,
    pub test_cascades: usize,
    pub observations: Option<usize>,
    pub brier_score: Option<f64>,
    pub log_loss: Option<f64>,
    pub reason: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ForecastPoint {
    pub step: usize,
    pub expected_active: f64,
    pub lower_active: usize,
    pub upper_active: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Forecast {
    pub seed: u64,
    pub seed_nodes: Vec<String>,
    pub simulations: usize,
    pub horizon: usize,
    pub probability: f64,
    pub blocked_nodes: Vec<String>,
    pub transmission_multiplier: f64,
    pub points: Vec<ForecastPoint>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Analysis {
    pub data_digest: String,
    pub node_count: usize,
    pub edge_count: usize,
    pub cascade_count: usize,
    pub activation_count: usize,
    pub calibration: Calibration,
    pub backtest: Backtest,
    pub baseline: Forecast,
    pub intervention: Forecast,
    pub receipt_digest: String,
}

impl Analysis {
    pub fn to_json(&self) -> String {
        let unsigned = render_analysis(self);
        let mut output = unsigned;
        output.pop();
        output.push_str(&format!(",\"receipt_digest\":{}}}", quoted(&self.receipt_digest)));
        output
    }

    pub fn verify_receipt(&self) -> bool {
        self.receipt_digest == format!("sha256:{}", sha256_hex(render_analysis(self).as_bytes()))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DiffusionError {
    Invalid(String),
    DeadlineExceeded,
}

impl Display for DiffusionError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Invalid(message) => formatter.write_str(message),
            Self::DeadlineExceeded => formatter.write_str("analysis deadline exceeded"),
        }
    }
}

impl std::error::Error for DiffusionError {}

pub fn analyze(input: Input, config: Config, limits: &Limits) -> Result<Analysis, DiffusionError> {
    validate_limits(limits)?;
    if config.max_runtime.is_zero() || config.max_runtime > MAX_RUNTIME {
        return Err(invalid("max runtime must be greater than zero and at most 600 seconds"));
    }
    let deadline = Instant::now()
        .checked_add(config.max_runtime)
        .ok_or_else(|| invalid("max runtime overflows the monotonic clock"))?;
    let graph = validate(input, &config, limits)?;
    let observations = observations(&graph.parents, &graph.cascades, limits, deadline)?;
    let calibration = calibrate(&observations, deadline)?;
    let backtest = backtest(&graph.parents, &graph.cascades, limits, deadline)?;
    let baseline = forecast(
        &graph.adjacency,
        &graph.seeds,
        config.horizon,
        config.simulations,
        config.random_seed,
        calibration.probability,
        &[],
        1.0,
        deadline,
    )?;
    let intervention = forecast(
        &graph.adjacency,
        &graph.seeds,
        config.horizon,
        config.simulations,
        config.random_seed,
        calibration.probability,
        &graph.blocked_nodes,
        config.transmission_multiplier,
        deadline,
    )?;
    let data_digest = format!("sha256:{}", sha256_hex(canonical_input(&graph).as_bytes()));
    let mut analysis = Analysis {
        data_digest,
        node_count: graph.nodes.len(),
        edge_count: graph.edges.len(),
        cascade_count: graph.cascades.len(),
        activation_count: graph.cascades.iter().map(|cascade| cascade.activations.len()).sum(),
        calibration,
        backtest,
        baseline,
        intervention,
        receipt_digest: String::new(),
    };
    analysis.receipt_digest =
        format!("sha256:{}", sha256_hex(render_analysis(&analysis).as_bytes()));
    Ok(analysis)
}

struct Graph {
    nodes: Vec<String>,
    edges: Vec<Edge>,
    cascades: Vec<Cascade>,
    seeds: Vec<String>,
    blocked_nodes: Vec<String>,
    adjacency: BTreeMap<String, Vec<String>>,
    parents: BTreeMap<String, Vec<String>>,
}

fn validate(mut input: Input, config: &Config, limits: &Limits) -> Result<Graph, DiffusionError> {
    if input.nodes.is_empty() || input.nodes.len() > limits.max_nodes {
        return Err(invalid("node count is outside the configured bound"));
    }
    if input.edges.len() > limits.max_edges {
        return Err(invalid("edge count exceeds the configured bound"));
    }
    if input.cascades.is_empty() || input.cascades.len() > limits.max_cascades {
        return Err(invalid("cascade count is outside the configured bound"));
    }
    if !(1..=limits.max_horizon).contains(&config.horizon) {
        return Err(invalid("horizon is outside the configured bound"));
    }
    if !(1..=limits.max_simulations).contains(&config.simulations) {
        return Err(invalid("simulation count is outside the configured bound"));
    }
    if !config.transmission_multiplier.is_finite()
        || !(0.0..=10.0).contains(&config.transmission_multiplier)
    {
        return Err(invalid("transmission multiplier must be finite and from 0 to 10"));
    }
    input.nodes.iter_mut().try_for_each(|node| normalize_id(node, "node"))?;
    input.nodes.sort();
    if input.nodes.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err(invalid("node ids must be unique"));
    }
    let nodes: BTreeSet<_> = input.nodes.iter().cloned().collect();
    if input.seeds.is_empty() {
        return Err(invalid("at least one seed is required"));
    }
    input.seeds.iter_mut().try_for_each(|node| normalize_id(node, "seed"))?;
    input.seeds.sort();
    if input.seeds.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err(invalid("seed node ids must be unique"));
    }
    let mut blocked = config.blocked_nodes.clone();
    blocked.iter_mut().try_for_each(|node| normalize_id(node, "blocked node"))?;
    blocked.sort();
    if blocked.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err(invalid("blocked node ids must be unique"));
    }
    if input.seeds.iter().chain(blocked.iter()).any(|node| !nodes.contains(node)) {
        return Err(invalid("seeds and blocked nodes must exist in the graph"));
    }
    if input.seeds.iter().any(|seed| blocked.binary_search(seed).is_ok()) {
        return Err(invalid("seed nodes cannot be blocked"));
    }

    for edge in &mut input.edges {
        normalize_id(&mut edge.source, "edge source")?;
        normalize_id(&mut edge.target, "edge target")?;
        if edge.source == edge.target {
            return Err(invalid("self edges are not supported"));
        }
        if !nodes.contains(&edge.source) || !nodes.contains(&edge.target) {
            return Err(invalid("edge endpoint is not a known node"));
        }
    }
    input.edges.sort();
    if input.edges.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err(invalid("directed edges must be unique"));
    }
    let mut activations = 0usize;
    for cascade in &mut input.cascades {
        normalize_id(&mut cascade.cascade_id, "cascade id")?;
        if !valid_utc_timestamp(&cascade.started_at) {
            return Err(invalid("started_at must be canonical UTC RFC3339"));
        }
        if !(1..=limits.max_observation_step).contains(&cascade.observation_end_step) {
            return Err(invalid("observation window is outside the configured bound"));
        }
        if cascade.activations.is_empty() {
            return Err(invalid("each cascade needs at least one activation"));
        }
        for activation in &mut cascade.activations {
            normalize_id(&mut activation.node, "activation node")?;
            if !nodes.contains(&activation.node) {
                return Err(invalid("activation references an unknown node"));
            }
            if activation.step > cascade.observation_end_step {
                return Err(invalid("activation occurs after its observation window"));
            }
        }
        cascade.activations.sort_by(|a, b| (a.step, &a.node).cmp(&(b.step, &b.node)));
        let unique: BTreeSet<_> = cascade.activations.iter().map(|item| &item.node).collect();
        if unique.len() != cascade.activations.len() {
            return Err(invalid("a node may activate only once per cascade"));
        }
        activations = activations
            .checked_add(cascade.activations.len())
            .ok_or_else(|| invalid("activation count overflow"))?;
        if activations > limits.max_activations {
            return Err(invalid("activation count exceeds the configured bound"));
        }
    }
    let cascade_ids =
        input.cascades.iter().map(|cascade| &cascade.cascade_id).collect::<BTreeSet<_>>();
    if cascade_ids.len() != input.cascades.len() {
        return Err(invalid("cascade ids must be unique"));
    }
    input
        .cascades
        .sort_by(|a, b| (&a.started_at, &a.cascade_id).cmp(&(&b.started_at, &b.cascade_id)));
    let mut adjacency =
        input.nodes.iter().map(|node| (node.clone(), Vec::new())).collect::<BTreeMap<_, _>>();
    let mut parents = adjacency.clone();
    for edge in &input.edges {
        adjacency.get_mut(&edge.source).expect("validated source").push(edge.target.clone());
        parents.get_mut(&edge.target).expect("validated target").push(edge.source.clone());
    }
    Ok(Graph {
        nodes: input.nodes,
        edges: input.edges,
        cascades: input.cascades,
        seeds: input.seeds,
        blocked_nodes: blocked,
        adjacency,
        parents,
    })
}

fn validate_limits(limits: &Limits) -> Result<(), DiffusionError> {
    let hard = Limits::default();
    let valid = limits.max_nodes > 0
        && limits.max_nodes <= hard.max_nodes
        && limits.max_edges > 0
        && limits.max_edges <= hard.max_edges
        && limits.max_cascades > 0
        && limits.max_cascades <= hard.max_cascades
        && limits.max_activations > 0
        && limits.max_activations <= hard.max_activations
        && limits.max_observation_step > 0
        && limits.max_observation_step <= hard.max_observation_step
        && limits.max_calibration_observations > 0
        && limits.max_calibration_observations <= hard.max_calibration_observations
        && limits.max_horizon > 0
        && limits.max_horizon <= hard.max_horizon
        && limits.max_simulations > 0
        && limits.max_simulations <= hard.max_simulations;
    if valid {
        Ok(())
    } else {
        Err(invalid("limits must be positive and cannot exceed the application hard caps"))
    }
}

fn observations(
    parents: &BTreeMap<String, Vec<String>>,
    cascades: &[Cascade],
    limits: &Limits,
    deadline: Instant,
) -> Result<Vec<(usize, bool)>, DiffusionError> {
    let mut output = Vec::new();
    let mut scanned = 0usize;
    for cascade in cascades {
        check_deadline(deadline)?;
        let steps = cascade
            .activations
            .iter()
            .map(|item| (item.node.as_str(), item.step))
            .collect::<BTreeMap<_, _>>();
        let start = cascade.activations.iter().map(|item| item.step).min().expect("non-empty");
        for step in start..cascade.observation_end_step {
            if step % 32 == 0 {
                check_deadline(deadline)?;
            }
            for (target, sources) in parents {
                scanned += 1;
                if scanned > limits.max_calibration_observations.saturating_mul(4) {
                    return Err(invalid("calibration exceeds the bounded graph-step budget"));
                }
                if steps.get(target.as_str()).is_some_and(|active| *active <= step) {
                    continue;
                }
                let exposure = sources
                    .iter()
                    .filter(|source| {
                        steps.get(source.as_str()).is_some_and(|active| *active == step)
                    })
                    .count();
                if exposure > 0 {
                    output.push((
                        exposure,
                        steps.get(target.as_str()).is_some_and(|active| *active == step + 1),
                    ));
                    if output.len() > limits.max_calibration_observations {
                        return Err(invalid(
                            "calibration observation count exceeds the configured bound",
                        ));
                    }
                }
            }
        }
    }
    if output.is_empty() {
        return Err(invalid("cascades contain no edge exposure observations"));
    }
    Ok(output)
}

fn calibrate(
    observations: &[(usize, bool)],
    deadline: Instant,
) -> Result<Calibration, DiffusionError> {
    let mut bins = BTreeMap::<usize, (usize, usize)>::new();
    for (index, &(exposure, positive)) in observations.iter().enumerate() {
        if index % 16_384 == 0 {
            check_deadline(deadline)?;
        }
        let counts = bins.entry(exposure).or_default();
        if positive {
            counts.0 += 1;
        } else {
            counts.1 += 1;
        }
    }
    let (mut low, mut high) = (1e-8, 1.0 - 1e-8);
    let ratio = (5.0_f64.sqrt() - 1.0) / 2.0;
    let mut left = high - ratio * (high - low);
    let mut right = low + ratio * (high - low);
    for _ in 0..96 {
        check_deadline(deadline)?;
        if nll_bins(left, &bins) < nll_bins(right, &bins) {
            high = right;
            right = left;
            left = high - ratio * (high - low);
        } else {
            low = left;
            left = right;
            right = low + ratio * (high - low);
        }
    }
    let probability = (low + high) / 2.0;
    let mut brier = 0.0;
    let mut information = 0.0;
    for (&exposure, &(positives, negatives)) in &bins {
        let prediction = activation_probability(probability, exposure);
        brier += positives as f64 * (prediction - 1.0).powi(2);
        brier += negatives as f64 * prediction.powi(2);
        let k = exposure as f64;
        let derivative = k * (1.0 - probability).powi(exposure.saturating_sub(1) as i32);
        if positives > 0 {
            let second =
                -k * (k - 1.0) * (1.0 - probability).powi(exposure.saturating_sub(2) as i32);
            information +=
                positives as f64 * (-second / prediction + (derivative / prediction).powi(2));
        }
        information += negatives as f64 * k / (1.0 - probability).powi(2);
    }
    let error = if information > 0.0 { 1.0 / information.sqrt() } else { 0.0 };
    Ok(Calibration {
        probability,
        confidence_low: (probability - 1.96 * error).max(0.0),
        confidence_high: (probability + 1.96 * error).min(1.0),
        observations: observations.len(),
        positives: bins.values().map(|(positives, _)| *positives).sum(),
        negative_log_likelihood: nll_bins(probability, &bins),
        brier_score: brier / observations.len() as f64,
    })
}

fn backtest(
    parents: &BTreeMap<String, Vec<String>>,
    cascades: &[Cascade],
    limits: &Limits,
    deadline: Instant,
) -> Result<Backtest, DiffusionError> {
    if cascades.len() < 3 {
        return Ok(Backtest {
            status: "unmeasured",
            train_cascades: cascades.len(),
            test_cascades: 0,
            observations: None,
            brier_score: None,
            log_loss: None,
            reason: Some("at least three cascades are required".into()),
        });
    }
    let split = ((cascades.len() * 4) / 5).clamp(1, cascades.len() - 1);
    let result = observations(parents, &cascades[..split], limits, deadline).and_then(|train| {
        let calibration = calibrate(&train, deadline)?;
        observations(parents, &cascades[split..], limits, deadline).map(|test| (calibration, test))
    });
    match result {
        Ok((calibration, test)) => {
            let mut brier = 0.0;
            for (index, &(exposure, positive)) in test.iter().enumerate() {
                if index % 16_384 == 0 {
                    check_deadline(deadline)?;
                }
                let observed = if positive { 1.0 } else { 0.0 };
                brier +=
                    (activation_probability(calibration.probability, exposure) - observed).powi(2);
            }
            let log_loss = nll_checked(calibration.probability, &test, deadline)?;
            Ok(Backtest {
                status: "measured",
                train_cascades: split,
                test_cascades: cascades.len() - split,
                observations: Some(test.len()),
                brier_score: Some(brier / test.len() as f64),
                log_loss: Some(log_loss / test.len() as f64),
                reason: None,
            })
        }
        Err(DiffusionError::DeadlineExceeded) => Err(DiffusionError::DeadlineExceeded),
        Err(error) => Ok(Backtest {
            status: "unmeasured",
            train_cascades: split,
            test_cascades: cascades.len() - split,
            observations: None,
            brier_score: None,
            log_loss: None,
            reason: Some(error.to_string()),
        }),
    }
}

fn forecast(
    adjacency: &BTreeMap<String, Vec<String>>,
    seeds: &[String],
    horizon: usize,
    simulations: usize,
    random_seed: u64,
    probability: f64,
    blocked: &[String],
    multiplier: f64,
    deadline: Instant,
) -> Result<Forecast, DiffusionError> {
    let blocked_set = blocked.iter().collect::<BTreeSet<_>>();
    let effective = (probability * multiplier).min(1.0);
    let mut totals = (0..=horizon).map(|_| Vec::with_capacity(simulations)).collect::<Vec<_>>();
    for run in 0..simulations {
        if run % 32 == 0 {
            check_deadline(deadline)?;
        }
        let mut rng = SplitMix64::new(random_seed ^ run as u64);
        let mut active = seeds
            .iter()
            .filter(|node| !blocked_set.contains(node))
            .cloned()
            .collect::<BTreeSet<_>>();
        let mut frontier = active.iter().cloned().collect::<Vec<_>>();
        totals[0].push(active.len());
        for step in 1..=horizon {
            if step % 8 == 0 {
                check_deadline(deadline)?;
            }
            let mut newly = BTreeSet::new();
            for source in &frontier {
                for target in &adjacency[source] {
                    if active.contains(target)
                        || blocked_set.contains(target)
                        || newly.contains(target)
                    {
                        continue;
                    }
                    if rng.uniform() < effective {
                        newly.insert(target.clone());
                    }
                }
            }
            active.extend(newly.iter().cloned());
            frontier = newly.into_iter().collect();
            totals[step].push(active.len());
        }
    }
    let points = totals
        .into_iter()
        .enumerate()
        .map(|(step, mut values)| {
            values.sort_unstable();
            ForecastPoint {
                step,
                expected_active: values.iter().sum::<usize>() as f64 / values.len() as f64,
                lower_active: quantile(&values, 5, 100),
                upper_active: quantile(&values, 95, 100),
            }
        })
        .collect();
    Ok(Forecast {
        seed: random_seed,
        seed_nodes: seeds.to_vec(),
        simulations,
        horizon,
        probability: effective,
        blocked_nodes: blocked.to_vec(),
        transmission_multiplier: multiplier,
        points,
    })
}

fn activation_probability(probability: f64, exposure: usize) -> f64 {
    (1.0 - (1.0 - probability).powi(exposure as i32)).clamp(1e-15, 1.0 - 1e-15)
}
fn nll_checked(
    probability: f64,
    observations: &[(usize, bool)],
    deadline: Instant,
) -> Result<f64, DiffusionError> {
    let mut total = 0.0;
    for (index, &(exposure, positive)) in observations.iter().enumerate() {
        if index % 16_384 == 0 {
            check_deadline(deadline)?;
        }
        let prediction = activation_probability(probability, exposure);
        total -= (if positive { prediction } else { 1.0 - prediction }).ln();
    }
    Ok(total)
}
fn nll_bins(probability: f64, bins: &BTreeMap<usize, (usize, usize)>) -> f64 {
    bins.iter()
        .map(|(&exposure, &(positives, negatives))| {
            let prediction = activation_probability(probability, exposure);
            -(positives as f64) * prediction.ln() - (negatives as f64) * (1.0 - prediction).ln()
        })
        .sum()
}
fn quantile(values: &[usize], numerator: usize, denominator: usize) -> usize {
    values[((values.len() - 1) * numerator + denominator / 2) / denominator]
}
fn check_deadline(deadline: Instant) -> Result<(), DiffusionError> {
    if Instant::now() >= deadline { Err(DiffusionError::DeadlineExceeded) } else { Ok(()) }
}
fn invalid(message: impl Into<String>) -> DiffusionError {
    DiffusionError::Invalid(message.into())
}
fn normalize_id(value: &mut String, label: &str) -> Result<(), DiffusionError> {
    *value = value.trim().to_owned();
    if value.is_empty()
        || value.len() > 128
        || value.chars().any(|c| matches!(c, '\t' | '\r' | '\n') || c.is_control())
    {
        Err(invalid(format!("{label} must contain 1-128 safe characters")))
    } else {
        Ok(())
    }
}
fn valid_utc_timestamp(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.len() != 20
        || bytes[4] != b'-'
        || bytes[7] != b'-'
        || bytes[10] != b'T'
        || bytes[13] != b':'
        || bytes[16] != b':'
        || bytes[19] != b'Z'
    {
        return false;
    }
    if bytes.iter().enumerate().any(|(index, value)| {
        !matches!(index, 4 | 7 | 10 | 13 | 16 | 19) && !value.is_ascii_digit()
    }) {
        return false;
    }
    let parse = |start: usize, end: usize| {
        std::str::from_utf8(&bytes[start..end]).ok().and_then(|part| part.parse::<u32>().ok())
    };
    let (Some(year), Some(month), Some(day), Some(hour), Some(minute), Some(second)) =
        (parse(0, 4), parse(5, 7), parse(8, 10), parse(11, 13), parse(14, 16), parse(17, 19))
    else {
        return false;
    };
    let leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    let max_day = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap => 29,
        2 => 28,
        _ => return false,
    };
    year > 0 && (1..=max_day).contains(&day) && hour < 24 && minute < 60 && second < 60
}

fn canonical_input(graph: &Graph) -> String {
    let mut out = String::new();
    for node in &graph.nodes {
        out.push_str("N\t");
        out.push_str(node);
        out.push('\n');
    }
    for edge in &graph.edges {
        out.push_str("E\t");
        out.push_str(&edge.source);
        out.push('\t');
        out.push_str(&edge.target);
        out.push('\n');
    }
    for seed in &graph.seeds {
        out.push_str("S\t");
        out.push_str(seed);
        out.push('\n');
    }
    for cascade in &graph.cascades {
        out.push_str("C\t");
        out.push_str(&cascade.cascade_id);
        out.push('\t');
        out.push_str(&cascade.started_at);
        out.push('\t');
        out.push_str(&cascade.observation_end_step.to_string());
        out.push('\n');
        for activation in &cascade.activations {
            out.push_str("A\t");
            out.push_str(&cascade.cascade_id);
            out.push('\t');
            out.push_str(&activation.node);
            out.push('\t');
            out.push_str(&activation.step.to_string());
            out.push('\n');
        }
    }
    out
}

fn render_analysis(value: &Analysis) -> String {
    format!(
        "{{\"schema_version\":{},\"model_version\":{},\"data_digest\":{},\"data\":{{\"nodes\":{},\"edges\":{},\"cascades\":{},\"activations\":{}}},\"calibration\":{},\"backtest\":{},\"baseline\":{},\"intervention\":{},\"limitations\":[{},{},{},{}]}}",
        quoted(SCHEMA_VERSION),
        quoted(MODEL_VERSION),
        quoted(&value.data_digest),
        value.node_count,
        value.edge_count,
        value.cascade_count,
        value.activation_count,
        calibration_json(&value.calibration),
        backtest_json(&value.backtest),
        forecast_json(&value.baseline),
        forecast_json(&value.intervention),
        quoted("Candidate graph edges are observational paths, not causal proof."),
        quoted("The baseline uses one global synchronous independent-cascade probability."),
        quoted(
            "Forecast bands measure seeded process variation; parameter uncertainty is separate."
        ),
        quoted("Activation absence is interpreted only inside each explicit observation window.")
    )
}
fn calibration_json(c: &Calibration) -> String {
    format!(
        "{{\"model\":\"independent-cascade-global-mle\",\"confidence_method\":\"observed-information-normal-approximation\",\"transmission_probability\":{},\"confidence_low\":{},\"confidence_high\":{},\"observations\":{},\"positive_observations\":{},\"negative_log_likelihood\":{},\"brier_score\":{}}}",
        number(c.probability),
        number(c.confidence_low),
        number(c.confidence_high),
        c.observations,
        c.positives,
        number(c.negative_log_likelihood),
        number(c.brier_score)
    )
}
fn backtest_json(b: &Backtest) -> String {
    format!(
        "{{\"status\":{},\"train_cascades\":{},\"test_cascades\":{},\"observations\":{},\"brier_score\":{},\"log_loss\":{},\"reason\":{}}}",
        quoted(b.status),
        b.train_cascades,
        b.test_cascades,
        optional_usize(b.observations),
        optional_number(b.brier_score),
        optional_number(b.log_loss),
        b.reason.as_ref().map_or_else(|| "null".into(), |v| quoted(v))
    )
}
fn forecast_json(f: &Forecast) -> String {
    let points = f
        .points
        .iter()
        .map(|p| {
            format!(
                "{{\"step\":{},\"expected_active\":{},\"lower_active\":{},\"upper_active\":{}}}",
                p.step,
                number(p.expected_active),
                p.lower_active,
                p.upper_active
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let blocked = f.blocked_nodes.iter().map(|v| quoted(v)).collect::<Vec<_>>().join(",");
    let seeds = f.seed_nodes.iter().map(|v| quoted(v)).collect::<Vec<_>>().join(",");
    format!(
        "{{\"model\":\"independent-cascade\",\"model_version\":{},\"seed\":{},\"seed_nodes\":[{}],\"simulations\":{},\"horizon\":{},\"transmission_probability\":{},\"blocked_nodes\":[{}],\"transmission_multiplier\":{},\"points\":[{}]}}",
        quoted(MODEL_VERSION),
        f.seed,
        seeds,
        f.simulations,
        f.horizon,
        number(f.probability),
        blocked,
        number(f.transmission_multiplier),
        points
    )
}
fn number(value: f64) -> String {
    if value.is_finite() { format!("{value:.17e}") } else { "null".into() }
}
fn optional_number(value: Option<f64>) -> String {
    value.map_or_else(|| "null".into(), number)
}
fn optional_usize(value: Option<usize>) -> String {
    value.map_or_else(|| "null".into(), |v| v.to_string())
}
fn quoted(value: &str) -> String {
    let mut out = String::from("\"");
    for character in value.chars() {
        match character {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c < ' ' => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

struct SplitMix64 {
    state: u64,
}
impl SplitMix64 {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }
    fn uniform(&mut self) -> f64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut value = self.state;
        value = (value ^ (value >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        value = (value ^ (value >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        value ^= value >> 31;
        ((value >> 11) as f64) / ((1_u64 << 53) as f64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn cascade(id: &str, day: usize, positive: bool) -> Cascade {
        let mut activations = vec![Activation { node: "a".into(), step: 0 }];
        if positive {
            activations.push(Activation { node: "b".into(), step: 1 });
        }
        Cascade {
            cascade_id: id.into(),
            started_at: format!("2026-08-{day:02}T00:00:00Z"),
            observation_end_step: 2,
            activations,
        }
    }
    fn fixture() -> Input {
        Input {
            nodes: vec!["a".into(), "b".into(), "c".into()],
            edges: vec![
                Edge { source: "a".into(), target: "b".into() },
                Edge { source: "b".into(), target: "c".into() },
            ],
            cascades: vec![
                cascade("c1", 1, true),
                cascade("c2", 2, false),
                cascade("c3", 3, true),
                cascade("c4", 4, false),
                cascade("c5", 5, true),
            ],
            seeds: vec!["a".into()],
        }
    }
    #[test]
    fn deterministic_receipt_and_intervention() {
        let config = Config {
            blocked_nodes: vec!["b".into()],
            simulations: 128,
            horizon: 4,
            ..Config::default()
        };
        let first = analyze(fixture(), config.clone(), &Limits::default()).unwrap();
        let second = analyze(fixture(), config, &Limits::default()).unwrap();
        assert_eq!(first, second);
        assert!(first.verify_receipt());
        assert_eq!(first.intervention.points.last().unwrap().expected_active, 1.0);
        assert!(first.baseline.points.last().unwrap().expected_active > 1.0);
        assert_eq!(first.backtest.status, "measured");
    }
    #[test]
    fn duplicate_edges_and_invalid_deadlines_fail_closed() {
        let mut duplicate = fixture();
        duplicate.edges.push(duplicate.edges[0].clone());
        assert!(matches!(
            analyze(duplicate, Config::default(), &Limits::default()),
            Err(DiffusionError::Invalid(_))
        ));
        let config = Config { max_runtime: Duration::ZERO, ..Config::default() };
        assert!(matches!(
            analyze(fixture(), config, &Limits::default()),
            Err(DiffusionError::Invalid(_))
        ));
    }

    #[test]
    fn receipts_cover_seed_nodes_and_normalized_counts() {
        let first = analyze(fixture(), Config::default(), &Limits::default()).unwrap();
        let mut changed = fixture();
        changed.seeds = vec!["b".into()];
        let second = analyze(changed, Config::default(), &Limits::default()).unwrap();
        assert_ne!(first.data_digest, second.data_digest);
        assert_ne!(first.receipt_digest, second.receipt_digest);
        assert_eq!(first.cascade_count, 5);
        assert_eq!(first.activation_count, 8);
        assert!(first.to_json().contains("\"seed_nodes\":[\"a\"]"));
    }

    #[test]
    fn duplicate_cascades_and_relaxed_hard_caps_are_rejected() {
        let mut duplicate = fixture();
        duplicate.cascades[1].cascade_id = duplicate.cascades[0].cascade_id.clone();
        assert!(matches!(
            analyze(duplicate, Config::default(), &Limits::default()),
            Err(DiffusionError::Invalid(_))
        ));
        let limits = Limits { max_nodes: Limits::default().max_nodes + 1, ..Limits::default() };
        assert!(matches!(
            analyze(fixture(), Config::default(), &limits),
            Err(DiffusionError::Invalid(_))
        ));
    }
}
