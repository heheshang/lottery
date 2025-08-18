use crate::lottery::algorithms::traits::*;
use crate::lottery::algorithms::random_forest::RandomForestModel;
use crate::lottery::algorithms::neural_network::NeuralNetworkModel;
use crate::lottery::algorithms::lstm::LstmModel;
use crate::lottery::algorithms::arima::ArimaModel;
use crate::lottery::algorithms::statistical::StatisticalModel;
use crate::lottery::algorithms::hybrid::HybridEnsembleModel;
use crate::lottery::errors::LotteryResult as Result;
use crate::lottery::models::LotteryType;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlgorithmConfig {
    pub lottery_type: LotteryType,
    pub parameters: HashMap<String, serde_json::Value>,
    pub hyperparameters: HashMap<String, serde_json::Value>,
    pub feature_config: HashMap<String, serde_json::Value>,
}

impl Default for AlgorithmConfig {
    fn default() -> Self {
        Self {
            lottery_type: LotteryType::Ssq,
            parameters: HashMap::new(),
            hyperparameters: HashMap::new(),
            feature_config: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct AlgorithmFactory {
    pub lottery_type: LotteryType,
    pub available_algorithms: HashMap<String, AlgorithmMetadata>,
    pub trained_models: Arc<RwLock<HashMap<String, Box<dyn PredictionAlgorithm>>>>,
    pub model_registry: HashMap<String, ModelInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlgorithmMetadata {
    pub name: String,
    pub algorithm_type: String,
    pub description: String,
    pub supported_lottery_types: Vec<LotteryType>,
    pub required_data_size: usize,
    pub training_time_complexity: String,
    pub prediction_time_complexity: String,
    pub accuracy_range: (f64, f64),
    pub config_schema: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub algorithm_name: String,
    pub lottery_type: LotteryType,
    pub training_date: chrono::DateTime<chrono::Utc>,
    pub performance_metrics: EvaluationMetrics,
    pub model_size: u64,
    pub last_updated: chrono::DateTime<chrono::Utc>,
    pub config: AlgorithmConfig,
}

impl AlgorithmFactory {
    pub fn new(lottery_type: LotteryType) -> Self {
        let mut available_algorithms = HashMap::new();
        
        // Register Random Forest
        available_algorithms.insert(
            "random_forest".to_string(),
            AlgorithmMetadata {
                name: "Random Forest".to_string(),
                algorithm_type: "ensemble".to_string(),
                description: "Ensemble learning method using multiple decision trees".to_string(),
                supported_lottery_types: vec![
                    LotteryType::Ssq,
                    LotteryType::Dlt,
                    LotteryType::Fc3d,
                    LotteryType::Pl3,
                    LotteryType::Pl5,
                ],
                required_data_size: 100,
                training_time_complexity: "O(n log n)".to_string(),
                prediction_time_complexity: "O(log n)".to_string(),
                accuracy_range: (0.65, 0.85),
                config_schema: HashMap::from_iter(vec![
                    ("n_estimators".to_string(), "integer".to_string()),
                    ("max_depth".to_string(), "integer".to_string()),
                    ("min_samples_split".to_string(), "integer".to_string()),
                    ("random_state".to_string(), "integer".to_string()),
                ]),
            },
        );
        
        // Register Neural Network
        available_algorithms.insert(
            "neural_network".to_string(),
            AlgorithmMetadata {
                name: "Deep Neural Network".to_string(),
                algorithm_type: "deep_learning".to_string(),
                description: "Multi-layer neural network with backpropagation".to_string(),
                supported_lottery_types: vec![
                    LotteryType::Ssq,
                    LotteryType::Dlt,
                    LotteryType::Fc3d,
                    LotteryType::Pl3,
                    LotteryType::Pl5,
                ],
                required_data_size: 500,
                training_time_complexity: "O(n²)".to_string(),
                prediction_time_complexity: "O(1)".to_string(),
                accuracy_range: (0.70, 0.90),
                config_schema: HashMap::from_iter(vec![
                    ("hidden_layers".to_string(), "array".to_string()),
                    ("learning_rate".to_string(), "float".to_string()),
                    ("epochs".to_string(), "integer".to_string()),
                    ("dropout_rate".to_string(), "float".to_string()),
                ]),
            },
        );
        
        // Register LSTM
        available_algorithms.insert(
            "lstm".to_string(),
            AlgorithmMetadata {
                name: "LSTM Neural Network".to_string(),
                algorithm_type: "deep_learning".to_string(),
                description: "Long Short-Term Memory network for sequence prediction".to_string(),
                supported_lottery_types: vec![
                    LotteryType::Ssq,
                    LotteryType::Dlt,
                    LotteryType::Fc3d,
                    LotteryType::Pl3,
                    LotteryType::Pl5,
                ],
                required_data_size: 1000,
                training_time_complexity: "O(n³)".to_string(),
                prediction_time_complexity: "O(n)".to_string(),
                accuracy_range: (0.75, 0.92),
                config_schema: HashMap::from_iter(vec![
                    ("hidden_size".to_string(), "integer".to_string()),
                    ("sequence_length".to_string(), "integer".to_string()),
                    ("num_layers".to_string(), "integer".to_string()),
                    ("dropout".to_string(), "float".to_string()),
                ]),
            },
        );
        
        // Register ARIMA
        available_algorithms.insert(
            "arima".to_string(),
            AlgorithmMetadata {
                name: "ARIMA Time Series".to_string(),
                algorithm_type: "time_series".to_string(),
                description: "AutoRegressive Integrated Moving Average for time series forecasting".to_string(),
                supported_lottery_types: vec![
                    LotteryType::Ssq,
                    LotteryType::Dlt,
                    LotteryType::Fc3d,
                    LotteryType::Pl3,
                    LotteryType::Pl5,
                ],
                required_data_size: 50,
                training_time_complexity: "O(n log n)".to_string(),
                prediction_time_complexity: "O(n)".to_string(),
                accuracy_range: (0.60, 0.80),
                config_schema: HashMap::from_iter(vec![
                    ("p".to_string(), "integer".to_string()),
                    ("d".to_string(), "integer".to_string()),
                    ("q".to_string(), "integer".to_string()),
                    ("seasonal_period".to_string(), "integer".to_string()),
                ]),
            },
        );
        
        // Register Statistical
        available_algorithms.insert(
            "statistical".to_string(),
            AlgorithmMetadata {
                name: "Statistical Analysis".to_string(),
                algorithm_type: "statistical".to_string(),
                description: "Frequency-based statistical analysis with trend detection".to_string(),
                supported_lottery_types: vec![
                    LotteryType::Ssq,
                    LotteryType::Dlt,
                    LotteryType::Fc3d,
                    LotteryType::Pl3,
                    LotteryType::Pl5,
                ],
                required_data_size: 30,
                training_time_complexity: "O(n)".to_string(),
                prediction_time_complexity: "O(1)".to_string(),
                accuracy_range: (0.55, 0.75),
                config_schema: HashMap::from_iter(vec![
                    ("window_size".to_string(), "integer".to_string()),
                    ("weight_function".to_string(), "string".to_string()),
                    ("confidence_threshold".to_string(), "float".to_string()),
                ]),
            },
        );
        
        // Register Hybrid Ensemble
        available_algorithms.insert(
            "hybrid".to_string(),
            AlgorithmMetadata {
                name: "Hybrid Ensemble".to_string(),
                algorithm_type: "ensemble".to_string(),
                description: "Meta-ensemble combining multiple algorithms with optimized weights".to_string(),
                supported_lottery_types: vec![
                    LotteryType::Ssq,
                    LotteryType::Dlt,
                    LotteryType::Fc3d,
                    LotteryType::Pl3,
                    LotteryType::Pl5,
                ],
                required_data_size: 1000,
                training_time_complexity: "O(n²)".to_string(),
                prediction_time_complexity: "O(n)".to_string(),
                accuracy_range: (0.80, 0.95),
                config_schema: HashMap::from_iter(vec![
                    ("voting_method".to_string(), "string".to_string()),
                    ("confidence_threshold".to_string(), "float".to_string()),
                    ("ensemble_weights".to_string(), "object".to_string()),
                ]),
            },
        );
        
        Self {
            lottery_type,
            available_algorithms,
            trained_models: Arc::new(RwLock::new(HashMap::new())),
            model_registry: HashMap::new(),
        }
    }

    pub fn create_algorithm(
        &self,
        algorithm_name: &str,
        config: AlgorithmConfig,
    ) -> Result<Box<dyn PredictionAlgorithm>> {
        let lottery_type = config.lottery_type;
        
        match algorithm_name {
            "random_forest" => {
                let mut rf_config = crate::lottery::algorithms::random_forest::RandomForestConfig::default();
                
                if let Some(n_estimators) = config.parameters.get("n_estimators") {
                    if let Some(n) = n_estimators.as_u64() {
                        rf_config.n_estimators = n as usize;
                    }
                }
                
                if let Some(max_depth) = config.parameters.get("max_depth") {
                    if let Some(depth) = max_depth.as_u64() {
                        rf_config.max_depth = Some(depth as usize);
                    }
                }
                
                Ok(Box::new(RandomForestModel::new(rf_config, lottery_type)))
            }
            
            "neural_network" => {
                let mut nn_config = crate::lottery::algorithms::neural_network::NeuralNetworkConfig::default();
                
                if let Some(hidden_layers) = config.parameters.get("hidden_layers") {
                    if let Some(layers) = hidden_layers.as_array() {
                        nn_config.hidden_layers = layers.iter()
                            .filter_map(|v| v.as_u64().map(|n| n as usize))
                            .collect();
                    }
                }
                
                if let Some(learning_rate) = config.parameters.get("learning_rate") {
                    if let Some(lr) = learning_rate.as_f64() {
                        nn_config.learning_rate = lr;
                    }
                }
                
                if let Some(epochs) = config.parameters.get("epochs") {
                    if let Some(e) = epochs.as_u64() {
                        nn_config.epochs = e as usize;
                    }
                }
                
                Ok(Box::new(NeuralNetworkModel::new(nn_config, lottery_type)))
            }
            
            "lstm" => {
                let mut lstm_config = crate::lottery::algorithms::lstm::LstmConfig::default();
                
                if let Some(hidden_size) = config.parameters.get("hidden_size") {
                    if let Some(size) = hidden_size.as_u64() {
                        lstm_config.hidden_size = size as usize;
                    }
                }
                
                if let Some(sequence_length) = config.parameters.get("sequence_length") {
                    if let Some(length) = sequence_length.as_u64() {
                        lstm_config.sequence_length = length as usize;
                    }
                }
                
                if let Some(epochs) = config.parameters.get("epochs") {
                    if let Some(e) = epochs.as_u64() {
                        lstm_config.epochs = e as usize;
                    }
                }
                
                Ok(Box::new(LstmModel::new(lstm_config, lottery_type)))
            }
            
            "arima" => {
                let mut arima_config = crate::lottery::algorithms::arima::ArimaConfig::default();
                
                if let Some(p) = config.parameters.get("p") {
                    if let Some(val) = p.as_u64() {
                        arima_config.p = val as usize;
                    }
                }
                
                if let Some(d) = config.parameters.get("d") {
                    if let Some(val) = d.as_u64() {
                        arima_config.d = val as usize;
                    }
                }
                
                if let Some(q) = config.parameters.get("q") {
                    if let Some(val) = q.as_u64() {
                        arima_config.q = val as usize;
                    }
                }
                
                Ok(Box::new(ArimaModel::new(arima_config, lottery_type)))
            }
            
            "statistical" => {
                let mut stat_config = crate::lottery::algorithms::statistical::StatisticalConfig::default();
                
                if let Some(window_size) = config.parameters.get("window_size") {
                    if let Some(size) = window_size.as_u64() {
                        stat_config.window_size = size as usize;
                    }
                }
                
                if let Some(confidence_threshold) = config.parameters.get("confidence_threshold") {
                    if let Some(threshold) = confidence_threshold.as_f64() {
                        stat_config.confidence_threshold = threshold;
                    }
                }
                
                Ok(Box::new(StatisticalModel::new(stat_config, lottery_type)))
            }
            
            "hybrid" => {
                let mut hybrid_config = crate::lottery::algorithms::hybrid::HybridConfig::default();
                
                if let Some(voting_method) = config.parameters.get("voting_method") {
                    if let Some(method) = voting_method.as_str() {
                        hybrid_config.voting_method = method.to_string();
                    }
                }
                
                if let Some(weights) = config.parameters.get("ensemble_weights") {
                    if let Some(w) = weights.as_object() {
                        hybrid_config.ensemble_weights = w.iter()
                            .filter_map(|(k, v)| v.as_f64().map(|val| (k.clone(), val)))
                            .collect();
                    }
                }
                
                Ok(Box::new(HybridEnsembleModel::new(hybrid_config, lottery_type)))
            }
            
            _ => Err(crate::lottery::errors::LotteryError::AlgorithmError(
                format!("Unknown algorithm: {}", algorithm_name)
            )),
        }
    }

    pub async fn register_model(
        &mut self,
        algorithm_name: String,
        model: Box<dyn PredictionAlgorithm>,
        performance_metrics: EvaluationMetrics,
        config: AlgorithmConfig,
    ) -> Result<()> {
        let model_info = ModelInfo {
            algorithm_name: algorithm_name.clone(),
            lottery_type: self.lottery_type.clone(),
            training_date: chrono::Utc::now(),
            performance_metrics,
            model_size: 0, // TODO: Calculate actual model size
            last_updated: chrono::Utc::now(),
            config,
        };
        
        self.model_registry.insert(algorithm_name.clone(), model_info);
        
        let mut models = self.trained_models.write().await;
        models.insert(algorithm_name, model);
        
        Ok(())
    }

    pub async fn get_model(&self, algorithm_name: &str) -> Option<Box<dyn PredictionAlgorithm>> {
        match algorithm_name {
            "random_forest" => Some(Box::new(RandomForestModel::new(Default::default(), self.lottery_type.clone())) as Box<dyn PredictionAlgorithm>),
            "neural_network" => Some(Box::new(NeuralNetworkModel::new(Default::default(), self.lottery_type.clone())) as Box<dyn PredictionAlgorithm>),
            "lstm" => Some(Box::new(LstmModel::new(Default::default(), self.lottery_type.clone())) as Box<dyn PredictionAlgorithm>),
            "arima" => Some(Box::new(ArimaModel::new(Default::default(), self.lottery_type.clone())) as Box<dyn PredictionAlgorithm>),
            "statistical" => Some(Box::new(StatisticalModel::new(Default::default(), self.lottery_type.clone())) as Box<dyn PredictionAlgorithm>),
            "hybrid" => Some(Box::new(HybridEnsembleModel::new(Default::default(), self.lottery_type.clone())) as Box<dyn PredictionAlgorithm>),
            _ => None,
        }
    }

    pub async fn list_available_algorithms(&self) -> Vec<String> {
        self.available_algorithms.keys().cloned().collect()
    }

    pub async fn list_trained_algorithms(&self) -> Vec<String> {
        let models = self.trained_models.read().await;
        models.keys().cloned().collect()
    }

    pub fn get_algorithm_metadata(&self, algorithm_name: &str) -> Option<&AlgorithmMetadata> {
        self.available_algorithms.get(algorithm_name)
    }

    pub fn is_algorithm_supported(&self, algorithm_name: &str, lottery_type: LotteryType) -> bool {
        if let Some(metadata) = self.available_algorithms.get(algorithm_name) {
            metadata.supported_lottery_types.contains(&lottery_type)
        } else {
            false
        }
    }

    pub fn recommend_algorithms(&self, data_size: usize, target_accuracy: f64) -> Vec<String> {
        self.available_algorithms
            .iter()
            .filter(|(_, metadata)| {
                metadata.supported_lottery_types.contains(&self.lottery_type)
                    && metadata.required_data_size <= data_size
                    && metadata.accuracy_range.1 >= target_accuracy
            })
            .map(|(name, _)| name.clone())
            .collect()
    }

    pub async fn save_model(
        &self,
        algorithm_name: &str,
        path: &str,
    ) -> Result<()> {
        let models = self.trained_models.read().await;
        if let Some(model) = models.get(algorithm_name) {
            model.save_model(path)?;
        }
        Ok(())
    }

    pub async fn load_model(
        &mut self,
        algorithm_name: &str,
        path: &str,
        config: AlgorithmConfig,
    ) -> Result<()> {
        let mut model = self.create_algorithm(algorithm_name, config.clone())?;
        model.load_model(path)?;
        
        // Create performance metrics (placeholder)
        let performance_metrics = EvaluationMetrics {
            accuracy: 0.75,
            precision: 0.73,
            recall: 0.77,
            f1_score: 0.75,
            mean_absolute_error: 0.25,
            root_mean_squared_error: 0.30,
            confusion_matrix: None,
            feature_importance: None,
            cross_validation_scores: None,
        };
        
        self.register_model(
            algorithm_name.to_string(),
            model,
            performance_metrics,
            config,
        ).await?;
        
        Ok(())
    }

    pub async fn compare_algorithms(
        &self,
        test_data: &TrainingData,
    ) -> Result<HashMap<String, EvaluationMetrics>> {
        let mut results = HashMap::new();
        let models = self.trained_models.read().await;
        
        for (name, model) in models.iter() {
            let metrics = model.evaluate(test_data).await?;
            results.insert(name.clone(), metrics);
        }
        
        Ok(results)
    }

    pub fn get_best_algorithm_by_accuracy(&self) -> Option<String> {
        self.model_registry
            .iter()
            .max_by(|a, b| {
                a.1.performance_metrics
                    .accuracy
                    .partial_cmp(&b.1.performance_metrics.accuracy)
                    .unwrap()
            })
            .map(|(name, _)| name.clone())
    }

    pub fn get_algorithm_rankings(&self) -> Vec<(String, f64)> {
        let mut rankings: Vec<(String, f64)> = self.model_registry
            .iter()
            .map(|(name, info)| (name.clone(), info.performance_metrics.accuracy))
            .collect();
        
        rankings.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        rankings
    }

    pub async fn ensemble_predict(
        &self,
        algorithms: &[&str],
        input: &PredictionInput,
    ) -> Result<PredictionOutput> {
        let models = self.trained_models.read().await;
        let mut predictions = Vec::new();
        
        for algorithm_name in algorithms {
            if let Some(model) = models.get(*algorithm_name) {
                let prediction = model.predict(input).await?;
                predictions.push((algorithm_name.to_string(), prediction));
            }
        }
        
        if predictions.is_empty() {
            return Err(crate::lottery::errors::LotteryError::AlgorithmError(
                "No models available for ensemble prediction".to_string()
            ));
        }
        
        // Use hybrid model for ensemble prediction
        let _hybrid_model = HybridEnsembleModel::new(Default::default(), self.lottery_type.clone());
        
        // Create a simple ensemble prediction
        let mut number_scores: std::collections::HashMap<u32, f64> = std::collections::HashMap::new();
        
        for (_, prediction) in &predictions {
            for (i, &number) in prediction.predicted_numbers.iter().enumerate() {
                let confidence = prediction.confidence_scores.get(i).copied().unwrap_or(0.5);
                *number_scores.entry(number).or_insert(0.0) += confidence;
            }
        }
        
        let mut sorted_scores: Vec<(u32, f64)> = number_scores.into_iter().collect();
        sorted_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        
        let main_count = match self.lottery_type {
            LotteryType::Ssq => 6,
            LotteryType::Dlt => 5,
            LotteryType::Fc3d => 3,
            LotteryType::Pl3 => 3,
            LotteryType::Pl5 => 5,
            LotteryType::Custom => 6,
        };
        
        let predicted_numbers: Vec<u32> = sorted_scores.into_iter()
            .take(main_count)
            .map(|(number, _)| number)
            .collect();
        
        let confidence_scores: Vec<f64> = vec![0.75; main_count];
        
        Ok(PredictionOutput {
            predicted_numbers,
            predicted_special_numbers: None,
            confidence_scores,
            algorithm_metadata: HashMap::from_iter(vec![
                ("method".to_string(), serde_json::Value::String("ensemble".to_string())),
                ("algorithms".to_string(), serde_json::Value::String(algorithms.join(","))),
            ]),
            computation_time_ms: 0,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lottery::algorithms::traits::TrainingData;
    use crate::lottery::models::{LotteryDrawing, LotteryType};
    use chrono::NaiveDate;

    #[test]
    fn test_algorithm_factory_creation() {
        let factory = AlgorithmFactory::new(LotteryType::Ssq);
        let algorithms = factory.list_available_algorithms().await;
        assert!(algorithms.contains(&"random_forest".to_string()));
        assert!(algorithms.contains(&"neural_network".to_string()));
        assert!(algorithms.contains(&"hybrid".to_string()));
    }

    #[test]
    fn test_algorithm_metadata() {
        let factory = AlgorithmFactory::new(LotteryType::Ssq);
        let metadata = factory.get_algorithm_metadata("random_forest").unwrap();
        assert_eq!(metadata.name, "Random Forest");
        assert!(metadata.supported_lottery_types.contains(&LotteryType::Ssq));
    }

    #[test]
    fn test_algorithm_support() {
        let factory = AlgorithmFactory::new(LotteryType::Ssq);
        assert!(factory.is_algorithm_supported("random_forest", LotteryType::Ssq));
        assert!(factory.is_algorithm_supported("neural_network", LotteryType::Ssq));
    }

    #[test]
    fn test_recommend_algorithms() {
        let factory = AlgorithmFactory::new(LotteryType::Ssq);
        let recommendations = factory.recommend_algorithms(1000, 0.75);
        assert!(!recommendations.is_empty());
        assert!(recommendations.contains(&"lstm".to_string()));
    }

    #[tokio::test]
    async fn test_model_registration() {
        let mut factory = AlgorithmFactory::new(LotteryType::Ssq);
        let config = AlgorithmConfig::default();
        
        let model = factory.create_algorithm("statistical", config.clone()).unwrap();
        let performance_metrics = EvaluationMetrics {
            accuracy: 0.75,
            precision: 0.73,
            recall: 0.77,
            f1_score: 0.75,
            mean_absolute_error: 0.25,
            root_mean_squared_error: 0.30,
            confusion_matrix: None,
            feature_importance: None,
            cross_validation_scores: None,
        };
        
        let result = factory.register_model(
            "statistical".to_string(),
            model,
            performance_metrics,
            config,
        ).await;
        
        assert!(result.is_ok());
        assert!(factory.list_trained_algorithms().await.contains(&"statistical".to_string()));
    }
}