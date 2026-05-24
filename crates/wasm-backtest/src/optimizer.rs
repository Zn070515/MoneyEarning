use std::collections::HashMap;
use rand::Rng;
use serde::{Serialize, Deserialize};
use wasm_core::{DataFrame, BtResult};

use crate::BacktestConfig;

/// Parameter optimization configuration
#[derive(Debug, Clone)]
pub struct OptimizerConfig {
    pub method: OptimizerMethod,
    pub max_iterations: usize,
    pub target_metric: TargetMetric,
    pub early_stop_rounds: usize,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OptimizerMethod {
    GridSearch,
    GeneticAlgorithm,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TargetMetric {
    SharpeRatio,
    TotalReturn,
    CalmarRatio,
    SortinoRatio,
}

impl Default for OptimizerConfig {
    fn default() -> Self {
        OptimizerConfig {
            method: OptimizerMethod::GridSearch,
            max_iterations: 5000,
            target_metric: TargetMetric::SharpeRatio,
            early_stop_rounds: 50,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizerResult {
    pub best_params: HashMap<String, f64>,
    pub best_score: f64,
    pub best_result: BtResult,
    pub all_results: Vec<(HashMap<String, f64>, f64)>,
    pub iterations: usize,
    pub convergence_generation: usize,
}

/// Run parameter optimization
pub fn optimize(
    df: &DataFrame,
    strategy: &str,
    param_grid: &HashMap<String, (f64, f64, f64)>, // (min, max, step)
    config: &OptimizerConfig,
    bt_config: &BacktestConfig,
) -> OptimizerResult {
    match config.method {
        OptimizerMethod::GridSearch => grid_search_optimize(df, strategy, param_grid, config, bt_config),
        OptimizerMethod::GeneticAlgorithm => genetic_optimize(df, strategy, param_grid, config, bt_config),
    }
}

fn grid_search_optimize(
    df: &DataFrame,
    strategy: &str,
    param_grid: &HashMap<String, (f64, f64, f64)>,
    config: &OptimizerConfig,
    bt_config: &BacktestConfig,
) -> OptimizerResult {
    let combos = generate_grid_combinations(param_grid);
    let mut best_params = HashMap::new();
    let mut best_score = f64::NEG_INFINITY;
    let mut best_result: Option<BtResult> = None;
    let mut all_results = Vec::new();

    for (i, combo) in combos.iter().enumerate() {
        if i >= config.max_iterations { break; }

        let result = crate::run_with_template(df, strategy, combo, bt_config);
        let score = extract_metric(&result, config.target_metric);

        all_results.push((combo.clone(), score));

        if score > best_score {
            best_score = score;
            best_params = combo.clone();
            best_result = Some(result);
        }
    }

    OptimizerResult {
        best_params,
        best_score,
        best_result: best_result.unwrap_or_else(|| BtResult {
            total_return: 0.0, annual_return: 0.0, max_drawdown: 0.0,
            sharpe_ratio: 0.0, sortino_ratio: 0.0, calmar_ratio: 0.0,
            win_rate: 0.0, profit_loss_ratio: 0.0, total_trades: 0,
            equity_curve: vec![], monthly_returns: vec![],
            trades: vec![], max_drawdown_duration: 0, annual_volatility: 0.0,
        }),
        all_results,
        iterations: combos.len().min(config.max_iterations),
        convergence_generation: 0,
    }
}

fn generate_grid_combinations(grid: &HashMap<String, (f64, f64, f64)>) -> Vec<HashMap<String, f64>> {
    if grid.is_empty() {
        return vec![HashMap::new()];
    }

    let mut results = vec![HashMap::new()];
    for (key, &(min, max, step)) in grid.iter() {
        let values = generate_range(min, max, step);

        let mut new_results = Vec::new();
        for existing in &results {
            for &val in &values {
                let mut combo = existing.clone();
                combo.insert(key.clone(), val);
                new_results.push(combo);
            }
        }
        results = new_results;
    }
    results
}

fn generate_range(min: f64, max: f64, step: f64) -> Vec<f64> {
    if step <= 0.0 {
        return vec![(min + max) / 2.0];
    }
    let mut vals = Vec::new();
    let mut x = min;
    while x <= max + 1e-10 {
        vals.push((x * 1000.0).round() / 1000.0);
        x += step;
    }
    vals
}

/// Genetic algorithm for parameter optimization
fn genetic_optimize(
    df: &DataFrame,
    strategy: &str,
    param_grid: &HashMap<String, (f64, f64, f64)>,
    config: &OptimizerConfig,
    bt_config: &BacktestConfig,
) -> OptimizerResult {
    let pop_size = 50;
    let mutation_rate = 0.15;
    let crossover_rate = 0.70;
    let elite_count = 5;

    let keys: Vec<&String> = param_grid.keys().collect();
    if keys.is_empty() {
        return OptimizerResult {
            best_params: HashMap::new(),
            best_score: 0.0,
            best_result: BtResult {
                total_return: 0.0, annual_return: 0.0, max_drawdown: 0.0,
                sharpe_ratio: 0.0, sortino_ratio: 0.0, calmar_ratio: 0.0,
                win_rate: 0.0, profit_loss_ratio: 0.0, total_trades: 0,
                equity_curve: vec![], monthly_returns: vec![],
                trades: vec![], max_drawdown_duration: 0, annual_volatility: 0.0,
            },
            all_results: vec![],
            iterations: 0,
            convergence_generation: 0,
        };
    }

    let mut rng = rand::thread_rng();

    // Initialize population
    let mut population: Vec<(HashMap<String, f64>, f64)> = Vec::with_capacity(pop_size);
    for _ in 0..pop_size {
        let individual: HashMap<String, f64> = keys.iter().map(|k| {
            let &(min, max, step) = param_grid.get(k.as_str()).unwrap_or(&(0.0, 100.0, 1.0));
            let steps = ((max - min) / step) as i32;
            let s = if steps > 0 { rng.gen_range(0..=steps) } else { 0 };
            let val = (min + s as f64 * step).min(max);
            (k.to_string(), val)
        }).collect();
        let result = crate::run_with_template(df, strategy, &individual, bt_config);
        let score = extract_metric(&result, config.target_metric);
        population.push((individual, score));
    }

    population.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    let mut best_ever = population[0].clone();
    let mut no_improve = 0usize;
    let mut total_iters = pop_size;
    let mut convergence_gen = 0;

    for gen in 0..config.max_iterations / pop_size {
        // Elitism: keep top performers
        let mut new_pop: Vec<(HashMap<String, f64>, f64)> =
            population[..elite_count].to_vec();

        // Crossover and mutation
        while new_pop.len() < pop_size {
            let (p1_idx, p2_idx) = tournament_select(&population, &mut rng);

            let child = if rng.gen::<f64>() < crossover_rate {
                crossover(&population[p1_idx].0, &population[p2_idx].0, &keys, &mut rng)
            } else {
                population[p1_idx].0.clone()
            };

            let child = if rng.gen::<f64>() < mutation_rate {
                mutate(&child, param_grid, &keys, &mut rng)
            } else {
                child
            };

            let child = clamp_to_grid(&child, param_grid);
            let result = crate::run_with_template(df, strategy, &child, bt_config);
            let score = extract_metric(&result, config.target_metric);
            new_pop.push((child, score));
            total_iters += 1;
        }

        new_pop.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        population = new_pop;

        if population[0].1 > best_ever.1 {
            best_ever = population[0].clone();
            convergence_gen = gen;
            no_improve = 0;
        } else {
            no_improve += 1;
            if no_improve >= config.early_stop_rounds {
                break;
            }
        }

        if total_iters >= config.max_iterations { break; }
    }

    let result = crate::run_with_template(df, strategy, &best_ever.0, bt_config);
    let all_results: Vec<(HashMap<String, f64>, f64)> = population.into_iter().collect();

    OptimizerResult {
        best_params: best_ever.0,
        best_score: best_ever.1,
        best_result: result,
        all_results,
        iterations: total_iters,
        convergence_generation: convergence_gen,
    }
}

fn tournament_select(
    population: &[(HashMap<String, f64>, f64)],
    rng: &mut impl Rng,
) -> (usize, usize) {
    let tourney_size = 3;
    let n = population.len();

    let p1 = (0..tourney_size).map(|_| rng.gen_range(0..n))
        .max_by(|&a, &b| population[a].1.partial_cmp(&population[b].1).unwrap_or(std::cmp::Ordering::Equal))
        .unwrap_or(0);

    let p2 = (0..tourney_size).map(|_| rng.gen_range(0..n))
        .max_by(|&a, &b| population[a].1.partial_cmp(&population[b].1).unwrap_or(std::cmp::Ordering::Equal))
        .unwrap_or(0);

    (p1, p2)
}

fn crossover(
    parent1: &HashMap<String, f64>,
    parent2: &HashMap<String, f64>,
    keys: &[&String],
    rng: &mut impl Rng,
) -> HashMap<String, f64> {
    let mut child = HashMap::new();
    for k in keys {
        let alpha: f64 = rng.gen();
        let v1 = parent1.get(k.as_str()).copied().unwrap_or(0.0);
        let v2 = parent2.get(k.as_str()).copied().unwrap_or(0.0);
        child.insert(k.to_string(), alpha * v1 + (1.0 - alpha) * v2);
    }
    child
}

fn mutate(
    individual: &HashMap<String, f64>,
    param_grid: &HashMap<String, (f64, f64, f64)>,
    keys: &[&String],
    rng: &mut impl Rng,
) -> HashMap<String, f64> {
    let mut mutated = individual.clone();
    let idx = rng.gen_range(0..keys.len());
    let key = keys[idx];
    let &(min, max, step) = param_grid.get(key.as_str()).unwrap_or(&(0.0, 100.0, 1.0));
    let current = individual.get(key.as_str()).copied().unwrap_or((min + max) / 2.0);

    // Random walk mutation
    let delta = step * (rng.gen::<f64>() * 2.0 - 1.0) * 2.0;
    let new_val = (current + delta).clamp(min, max);
    // Round to step
    let new_val = (new_val / step).round() * step;
    mutated.insert(key.to_string(), new_val.clamp(min, max));
    mutated
}

fn clamp_to_grid(
    individual: &HashMap<String, f64>,
    param_grid: &HashMap<String, (f64, f64, f64)>,
) -> HashMap<String, f64> {
    let mut clamped = HashMap::new();
    for (k, v) in individual {
        if let Some((min, max, step)) = param_grid.get(k) {
            let rounded = (*v / step).round() * step;
            clamped.insert(k.clone(), rounded.clamp(*min, *max));
        } else {
            clamped.insert(k.clone(), *v);
        }
    }
    clamped
}

fn extract_metric(result: &BtResult, target: TargetMetric) -> f64 {
    match target {
        TargetMetric::SharpeRatio => result.sharpe_ratio,
        TargetMetric::TotalReturn => result.annual_return,
        TargetMetric::CalmarRatio => result.calmar_ratio,
        TargetMetric::SortinoRatio => result.sortino_ratio,
    }
}
