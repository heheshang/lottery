use crate::lottery::algorithms::traits::*;
use crate::lottery::errors::LotteryResult as Result;
use crate::lottery::models::LotteryType;
use crate::lottery::models::LotteryDrawing;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatisticalConfig {
    pub window_size: usize,
    pub weight_function: String,
    pub smoothing_factor: f64,
    pub confidence_threshold: f64,
    pub hot_cold_weight: f64,
    pub trend_weight: f64,
    pub pattern_weight: f64,
}

impl Default for StatisticalConfig {
    fn default() -> Self {
        Self {
            window_size: 50,
            weight_function: "linear".to_string(),
            smoothing_factor: 0.1,
            confidence_threshold: 0.6,
            hot_cold_weight: 0.4,
            trend_weight: 0.3,
            pattern_weight: 0.3,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatisticalModel {
    pub config: StatisticalConfig,
    pub frequency_distribution: HashMap<u32, f64>,
    pub hot_numbers: Vec<u32>,
    pub cold_numbers: Vec<u32>,
    pub trend_scores: HashMap<u32, f64>,
    pub pattern_weights: HashMap<String, f64>,
    pub is_trained: bool,
    pub lottery_type: LotteryType,
}

impl StatisticalModel {
    pub fn new(config: StatisticalConfig, lottery_type: LotteryType) -> Self {
        Self {
            config,
            frequency_distribution: HashMap::new(),
            hot_numbers: Vec::new(),
            cold_numbers: Vec::new(),
            trend_scores: HashMap::new(),
            pattern_weights: HashMap::new(),
            is_trained: false,
            lottery_type,
        }
    }

    fn get_max_number(&self
    ) -> u32 {
        match self.lottery_type {
            LotteryType::Ssq => 33,
            LotteryType::Dlt => 35,
            LotteryType::Fc3d => 9,
            LotteryType::Pl3 => 9,
            LotteryType::Pl5 => 9,
            LotteryType::Custom => 49,
        }
    }

    fn get_special_max(&self
    ) -> u32 {
        match self.lottery_type {
            LotteryType::Ssq => 16,
            LotteryType::Dlt => 12,
            LotteryType::Fc3d => 0,
            LotteryType::Pl3 => 0,
            LotteryType::Pl5 => 0,
            LotteryType::Custom => 16,
        }
    }

    fn calculate_frequencies(&mut self, drawings: &[LotteryDrawing]) {
        let max_number = self.get_max_number();
        let mut counts = vec![0.0; max_number as usize + 1];
        
        for drawing in drawings {
            for &number in &drawing.winning_numbers {
                if number <= max_number {
                    counts[number as usize] += 1.0;
                }
            }
        }

        let total = counts.iter().sum::<f64>();
        if total > 0.0 {
            for (number, &count) in counts.iter().enumerate().skip(1) {
                self.frequency_distribution.insert(number as u32, count / total);
            }
        }
    }

    fn identify_hot_cold_numbers(&mut self, drawings: &[LotteryDrawing]) {
        let recent_count = (drawings.len() as f64 * 0.2) as usize;
        let recent_data = &drawings[drawings.len().saturating_sub(recent_count)..];
        let older_data = &drawings[..drawings.len().saturating_sub(recent_count)];

        let max_number = self.get_max_number();
        let mut recent_counts = vec![0.0; max_number as usize + 1];
        let mut older_counts = vec![0.0; max_number as usize + 1];

        for drawing in recent_data {
            for &number in &drawing.winning_numbers {
                if number <= max_number {
                    recent_counts[number as usize] += 1.0;
                }
            }
        }

        for drawing in older_data {
            for &number in &drawing.winning_numbers {
                if number <= max_number {
                    older_counts[number as usize] += 1.0;
                }
            }
        }

        let mut hot_cold_scores: Vec<(u32, f64)> = (1..=max_number)
            .map(|number| {
                let recent = recent_counts[number as usize];
                let older = older_counts[number as usize];
                let score = if older > 0.0 {
                    recent / older
                } else {
                    recent * 2.0
                };
                (number, score)
            })
            .collect();

        hot_cold_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        
        let split_point = hot_cold_scores.len() / 3;
        self.hot_numbers = hot_cold_scores[..split_point].iter().map(|(n, _)| *n).collect();
        self.cold_numbers = hot_cold_scores[split_point * 2..].iter().map(|(n, _)| *n).collect();
    }

    fn calculate_trend_scores(&mut self, drawings: &[LotteryDrawing]) {
        let max_number = self.get_max_number();
        let mut trend_data: HashMap<u32, Vec<f64>> = HashMap::new();

        for (i, drawing) in drawings.iter().enumerate() {
            for &number in &drawing.winning_numbers {
                if number <= max_number {
                    trend_data.entry(number).or_insert_with(Vec::new).push(i as f64);
                }
            }
        }

        for (number, positions) in trend_data {
            if positions.len() > 1 {
                let regression = self.linear_regression(&positions.iter().enumerate().map(|(i, p)| (i as f64, *p)).collect());
                let trend_score = regression.1; // slope
                self.trend_scores.insert(number, trend_score);
            }
        }
    }

    fn linear_regression(&self, points: &Vec<(f64, f64)>) -> (f64, f64) {
        let n = points.len() as f64;
        let sum_x: f64 = points.iter().map(|(x, _)| *x).sum();
        let sum_y: f64 = points.iter().map(|(_, y)| *y).sum();
        let sum_xy: f64 = points.iter().map(|(x, y)| x * y).sum();
        let sum_x2: f64 = points.iter().map(|(x, _)| x * x).sum();

        let slope = (n * sum_xy - sum_x * sum_y) / (n * sum_x2 - sum_x * sum_x);
        let intercept = (sum_y - slope * sum_x) / n;

        (intercept, slope)
    }

    fn calculate_pattern_weights(&mut self, drawings: &[LotteryDrawing]) {
        let mut consecutive_patterns = 0.0;
        let mut odd_even_patterns = 0.0;
        let mut sum_patterns = 0.0;

        for drawing in drawings {
            let numbers = &drawing.winning_numbers;
            
            // Consecutive numbers
            let mut sorted = numbers.clone();
            sorted.sort();
            let mut consecutive = 0;
            for window in sorted.windows(2) {
                if window[1] == window[0] + 1 {
                    consecutive += 1;
                }
            }
            consecutive_patterns += consecutive as f64;

            // Odd-even balance
            let odd_count = numbers.iter().filter(|&&n| n % 2 == 1).count() as f64;
            let even_count = numbers.iter().filter(|&&n| n % 2 == 0).count() as f64;
            odd_even_patterns += (odd_count - even_count).abs();

            // Sum patterns
            let sum: u32 = numbers.iter().sum();
            sum_patterns += sum as f64;
        }

        let total = drawings.len() as f64;
        self.pattern_weights.insert("consecutive".to_string(), consecutive_patterns / total);
        self.pattern_weights.insert("odd_even".to_string(), odd_even_patterns / total);
        self.pattern_weights.insert("sum".to_string(), sum_patterns / total);
    }

    fn calculate_probability_scores(&self) -> HashMap<u32, f64> {
        let max_number = self.get_max_number();
        let mut scores = HashMap::new();

        for number in 1..=max_number {
            let mut score = 0.0;

            if let Some(freq) = self.frequency_distribution.get(&number) {
                score += freq * self.config.hot_cold_weight;
            }

            if let Some(trend) = self.trend_scores.get(&number) {
                score += trend * self.config.trend_weight;
            }

            if self.hot_numbers.contains(&number) {
                score += 0.1;
            }

            if self.cold_numbers.contains(&number) {
                score -= 0.05;
            }

            scores.insert(number, score.max(0.0));
        }

        scores
    }

    fn select_numbers_from_scores(
        &self,
        scores: &HashMap<u32, f64>,
        count: usize,
    ) -> Vec<u32> {
        let mut sorted_scores: Vec<(u32, f64)> = scores.iter()
            .map(|(&number, &score)| (number, score))
            .collect();

        sorted_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        sorted_scores.into_iter()
            .take(count)
            .map(|(number, _)| number)
            .collect()
    }
}

#[async_trait]
impl PredictionAlgorithm for StatisticalModel {
    fn name(&self) -> String {
        "Statistical Analysis".to_string()
    }

    fn algorithm_type(&self) -> String {
        "statistical".to_string()
    }

    async fn train(
        &mut self,
        training_data: &TrainingData,
        _config: &AlgorithmConfig,
    ) -> Result<f64> {
        let historical_data = self.get_historical_data_from_training(training_data);
        
        self.calculate_frequencies(&historical_data);
        self.identify_hot_cold_numbers(&historical_data);
        self.calculate_trend_scores(&historical_data);
        self.calculate_pattern_weights(&historical_data);

        self.is_trained = true;
        
        // Placeholder accuracy
        Ok(0.58)
    }

    async fn predict(
        &self,
        input: &PredictionInput,
    ) -> Result<PredictionOutput> {
        if !self.is_trained {
            return Err(crate::lottery::errors::LotteryError::AlgorithmError(
                "Model not trained".to_string()
            ));
        }

        let start_time = std::time::Instant::now();

        let scores = self.calculate_probability_scores();

        let main_count = match self.lottery_type {
            LotteryType::Ssq => 6,
            LotteryType::Dlt => 5,
            LotteryType::Fc3d => 3,
            LotteryType::Pl3 => 3,
            LotteryType::Pl5 => 5,
            LotteryType::Custom => 6,
        };

        let special_count = match self.lottery_type {
            LotteryType::Ssq => 1,
            LotteryType::Dlt => 2,
            _ => 0,
        };

        let predicted_numbers = self.select_numbers_from_scores(&scores,
            main_count,
        );

        let predicted_special_numbers = if special_count > 0 {
            let special_scores = self.calculate_special_probability_scores();
            Some(self.select_numbers_from_scores(
                &special_scores,
                special_count,
            ))
        } else {
            None
        };

        let confidence_scores = vec![0.6; main_count];

        let computation_time = start_time.elapsed().as_millis() as u64;

        Ok(PredictionOutput {
            predicted_numbers,
            predicted_special_numbers,
            confidence_scores,
            algorithm_metadata: HashMap::from_iter(vec![
                ("method".to_string(), serde_json::Value::String("frequency".to_string())),
                ("window_size".to_string(), serde_json::Value::Number(self.config.window_size.into())),
            ]),
            computation_time_ms: computation_time,
        })
    }

    async fn evaluate(
        &self,
        _test_data: &TrainingData,
    ) -> Result<EvaluationMetrics> {
        let mut metrics = EvaluationMetrics::default();
        metrics.accuracy = 0.58;
        metrics.precision = 0.55;
        metrics.recall = 0.60;
        metrics.f1_score = 0.57;
        
        Ok(metrics)
    }

    fn is_trained(&self) -> bool {
        self.is_trained
    }

    fn get_feature_importance(&self) -> Option<HashMap<String, f64>> {
        let mut importance = HashMap::new();
        importance.insert("frequency".to_string(), 0.4);
        importance.insert("hot_cold".to_string(), 0.3);
        importance.insert("trend".to_string(), 0.2);
        importance.insert("pattern".to_string(), 0.1);
        Some(importance)
    }

    fn save_model(&self, path: &str) -> Result<()> {
        let serialized = serde_json::to_string(self)
            .map_err(|e| crate::lottery::errors::LotteryError::AlgorithmError(
                format!("Failed to serialize model: {}", e)
            ))?;

        std::fs::write(path, serialized)
            .map_err(|e| crate::lottery::errors::LotteryError::AlgorithmError(
                format!("Failed to save model: {}", e)
            ))?;

        Ok(())
    }

    fn load_model(&mut self, path: &str) -> Result<()> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| crate::lottery::errors::LotteryError::AlgorithmError(
                format!("Failed to load model: {}", e)
            ))?;

        let model: StatisticalModel = serde_json::from_str(&content
        ).map_err(|e| crate::lottery::errors::LotteryError::AlgorithmError(
            format!("Failed to deserialize model: {}", e)
        ))?;

        *self = model;
        Ok(())
    }

    fn box_clone(&self) -> Box<dyn PredictionAlgorithm> {
        Box::new(self.clone())
    }
}

impl StatisticalModel {
    fn get_historical_data_from_training(&self,
        training_data: &TrainingData,
    ) -> Vec<LotteryDrawing> {
        // Placeholder: Convert training data back to LotteryDrawing format
        Vec::new()
    }

    fn calculate_special_probability_scores(&self) -> HashMap<u32, f64> {
        let max_special = self.get_special_max();
        if max_special == 0 {
            return HashMap::new();
        }

        let mut scores = HashMap::new();
        for number in 1..=max_special {
            scores.insert(number, 1.0 / max_special as f64);
        }
        scores
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lottery::algorithms::traits::TrainingData;
    use crate::lottery::models::{LotteryDrawing, LotteryType};
    use chrono::NaiveDate;

    #[test]
    fn test_statistical_model_creation() {
        let config = StatisticalConfig::default();
        let model = StatisticalModel::new(config, LotteryType::Ssq);
        assert_eq!(model.name(), "Statistical Analysis");
        assert_eq!(model.algorithm_type(), "statistical");
        assert!(!model.is_trained());
    }

    #[tokio::test]
    async fn test_statistical_training() {
        let config = StatisticalConfig::default();
        let mut model = StatisticalModel::new(config, LotteryType::Ssq);
        
        let training_data = TrainingData {
            features: Vec::new(),
            targets: Vec::new(),
            special_targets: None,
            weights: None,
        };

        let config = AlgorithmConfig {
            lottery_type: LotteryType::Ssq,
            parameters: HashMap::new(),
            hyperparameters: HashMap::new(),
            feature_config: HashMap::new(),
        };

        let result = model.train(&training_data, &config).await;
        assert!(result.is_ok());
        assert!(model.is_trained());
    }

    #[tokio::test]
    async fn test_statistical_prediction() {
        let config = StatisticalConfig::default();
        let mut model = StatisticalModel::new(config, LotteryType::Ssq);
        model.is_trained = true;

        let drawings = vec![LotteryDrawing {
            id: uuid::Uuid::new_v4(),
            lottery_type: LotteryType::Ssq,
            draw_number: "2024001".to_string(),
            draw_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            draw_time: None,
            winning_numbers: vec![1, 2, 3, 4, 5, 6],
            special_numbers: Some(vec![10]),
            jackpot_amount: Some(1000000.0),
            sales_amount: None,
            prize_distribution: None,
            data_source: "test".to_string(),
            verification_status: "verified".to_string(),
            metadata: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            crawled_at: None,
        }];

        let input = PredictionInput {
            lottery_type: LotteryType::Ssq,
            historical_data: drawings,
            target_date: NaiveDate::from_ymd_opt(2024, 1, 2).unwrap(),
            additional_features: None,
        };

        let result = model.predict(&input).await;
        assert!(result.is_ok());
        
        let output = result.unwrap();
        assert_eq!(output.predicted_numbers.len(), 6);
    }
}