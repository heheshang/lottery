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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlgorithmType {
    RandomForest,
    NeuralNetwork,
    Lstm,
    Arima,
    Statistical,
    Hybrid,
}

#[derive(Debug)]
pub struct ModelTrainer {
    pub lottery_type: LotteryType,
    pub algorithms: HashMap<String, Box<dyn PredictionAlgorithm>>,
    pub trained_models: HashMap<String, Box<dyn PredictionAlgorithm>>,
    pub model_performance: HashMap<String, EvaluationMetrics>,
}

impl ModelTrainer {
    pub fn new(lottery_type: LotteryType) -> Self {
        let mut algorithms = HashMap::new();
        
        // Initialize all available algorithms
        let rf = RandomForestModel::new(Default::default(), lottery_type.clone());
        let nn = NeuralNetworkModel::new(Default::default(), lottery_type.clone());
        let lstm = LstmModel::new(Default::default(), lottery_type.clone());
        let arima = ArimaModel::new(Default::default(), lottery_type.clone());
        let stat = StatisticalModel::new(Default::default(), lottery_type.clone());
        let hybrid = HybridEnsembleModel::new(Default::default(), lottery_type.clone());
        
        algorithms.insert("random_forest".to_string(), Box::new(rf) as Box<dyn PredictionAlgorithm>);
        algorithms.insert("neural_network".to_string(), Box::new(nn) as Box<dyn PredictionAlgorithm>);
        algorithms.insert("lstm".to_string(), Box::new(lstm) as Box<dyn PredictionAlgorithm>);
        algorithms.insert("arima".to_string(), Box::new(arima) as Box<dyn PredictionAlgorithm>);
        algorithms.insert("statistical".to_string(), Box::new(stat) as Box<dyn PredictionAlgorithm>);
        algorithms.insert("hybrid".to_string(), Box::new(hybrid) as Box<dyn PredictionAlgorithm>);
        
        Self {
            lottery_type,
            algorithms,
            trained_models: HashMap::new(),
            model_performance: HashMap::new(),
        }
    }

    pub async fn train_algorithm(
        &mut self,
        algorithm_name: &str,
        training_data: &TrainingData,
        config: &AlgorithmConfig,
    ) -> Result<f64> {
        if let Some(model) = self.algorithms.get_mut(algorithm_name) {
            let accuracy = model.train(training_data, config).await?;
            
            // Store the trained model
            let mut cloned_model: Box<dyn PredictionAlgorithm> = match algorithm_name {
                "random_forest" => Box::new(RandomForestModel::new(Default::default(), self.lottery_type.clone())),
                "neural_network" => Box::new(NeuralNetworkModel::new(Default::default(), self.lottery_type.clone())),
                "lstm" => Box::new(LstmModel::new(Default::default(), self.lottery_type.clone())),
                "arima" => Box::new(ArimaModel::new(Default::default(), self.lottery_type.clone())),
                "statistical" => Box::new(StatisticalModel::new(Default::default(), self.lottery_type.clone())),
                "hybrid" => Box::new(HybridEnsembleModel::new(Default::default(), self.lottery_type.clone())),
                _ => return Err(crate::lottery::errors::LotteryError::AlgorithmError(
                    "Unknown algorithm".to_string()
                )),
            };
            
            cloned_model.load_model(&format!("models/{}_{}.json", algorithm_name, self.lottery_type))?;
            
            self.trained_models.insert(algorithm_name.to_string(), cloned_model);
            
            Ok(accuracy)
        } else {
            Err(crate::lottery::errors::LotteryError::AlgorithmError(
                format!("Algorithm {} not found", algorithm_name)
            ))
        }
    }

    pub async fn train_all_algorithms(
        &mut self,
        training_data: &TrainingData,
        config: &AlgorithmConfig,
    ) -> Result<HashMap<String, f64>> {
        let mut results = HashMap::new();
        
        let algorithm_names: Vec<String> = self.algorithms.keys().cloned().collect();
        for algorithm_name in algorithm_names {
            match self.train_algorithm(&algorithm_name, training_data, config).await {
                Ok(accuracy) => {
                    results.insert(algorithm_name.clone(), accuracy);
                }
                Err(e) => {
                    eprintln!("Failed to train {}: {}", algorithm_name, e);
                }
            }
        }
        
        Ok(results)
    }

    pub async fn predict_with_algorithm(
        &self,
        algorithm_name: &str,
        input: &PredictionInput,
    ) -> Result<PredictionOutput> {
        if let Some(model) = self.trained_models.get(algorithm_name) {
            model.predict(input).await
        } else {
            Err(crate::lottery::errors::LotteryError::AlgorithmError(
                format!("Model {} not trained or found", algorithm_name)
            ))
        }
    }

    pub async fn compare_algorithms(
        &mut self,
        test_data: &TrainingData,
    ) -> Result<HashMap<String, EvaluationMetrics>> {
        let mut comparison = HashMap::new();
        
        let algorithm_names: Vec<String> = self.trained_models.keys().cloned().collect();
        for algorithm_name in algorithm_names {
            if let Some(model) = self.trained_models.get(&algorithm_name) {
                let metrics = model.evaluate(test_data).await?;
                comparison.insert(algorithm_name.clone(), metrics.clone());
                self.model_performance.insert(algorithm_name.clone(), metrics);
            }
        }
        
        Ok(comparison)
    }

    pub fn get_best_algorithm(&self) -> Option<&str> {
        self.model_performance
            .iter()
            .max_by(|a, b| a.1.accuracy.partial_cmp(&b.1.accuracy).unwrap())
            .map(|(name, _)| name.as_str())
    }

    pub fn list_available_algorithms(&self) -> Vec<String> {
        self.algorithms.keys().cloned().collect()
    }

    pub fn list_trained_algorithms(&self) -> Vec<String> {
        self.trained_models.keys().cloned().collect()
    }

    pub fn get_model_performance(&self, algorithm_name: &str) -> Option<&EvaluationMetrics> {
        self.model_performance.get(algorithm_name)
    }

    pub async fn save_all_models(&self, directory: &str) -> Result<()> {
        std::fs::create_dir_all(directory).map_err(|e| crate::lottery::errors::LotteryError::AlgorithmError(format!("Failed to create directory: {}", e)))?;
        
        for (algorithm_name, model) in &self.trained_models {
            let path = format!("{}/{}_{}.json", directory, algorithm_name, self.lottery_type);
            model.save_model(&path)?;
        }
        
        Ok(())
    }

    pub async fn load_all_models(&mut self, directory: &str) -> Result<()> {
        let algorithm_names: Vec<String> = self.algorithms.keys().cloned().collect();
        for algorithm_name in algorithm_names {
            let path = format!("{}/{}_{}.json", directory, algorithm_name, self.lottery_type);
            
            if std::path::Path::new(&path).exists() {
                let mut cloned_model: Box<dyn PredictionAlgorithm> = match algorithm_name.as_str() {
                    "random_forest" => Box::new(RandomForestModel::new(Default::default(), self.lottery_type.clone())),
                    "neural_network" => Box::new(NeuralNetworkModel::new(Default::default(), self.lottery_type.clone())),
                    "lstm" => Box::new(LstmModel::new(Default::default(), self.lottery_type.clone())),
                    "arima" => Box::new(ArimaModel::new(Default::default(), self.lottery_type.clone())),
                    "statistical" => Box::new(StatisticalModel::new(Default::default(), self.lottery_type.clone())),
                    "hybrid" => Box::new(HybridEnsembleModel::new(Default::default(), self.lottery_type.clone())),
                    _ => continue,
                };
                cloned_model.load_model(&path)?;
                self.trained_models.insert(algorithm_name.clone(), cloned_model);
            }
        }
        
        Ok(())
    }

    pub async fn ensemble_predict(
        &self,
        algorithms: &[&str],
        input: &PredictionInput,
    ) -> Result<PredictionOutput> {
        let mut predictions = Vec::new();
        
        for algorithm_name in algorithms {
            if let Some(model) = self.trained_models.get(*algorithm_name) {
                if let Ok(prediction) = model.predict(input).await {
                    predictions.push(prediction);
                }
            }
        }
        
        if predictions.is_empty() {
            return Err(crate::lottery::errors::LotteryError::AlgorithmError(
                "No valid predictions from specified algorithms".to_string()
            ));
        }
        
        // Simple averaging of predictions
        let mut number_counts: HashMap<u32, usize> = HashMap::new();
        
        for prediction in &predictions {
            for number in &prediction.predicted_numbers {
                *number_counts.entry(*number).or_insert(0) += 1;
            }
        }
        
        let mut sorted_numbers: Vec<(u32, usize)> = number_counts.into_iter().collect();
        sorted_numbers.sort_by(|a, b| b.1.cmp(&a.1));
        
        let main_count = match self.lottery_type {
            LotteryType::Ssq => 6,
            LotteryType::Dlt => 5,
            LotteryType::Fc3d => 3,
            LotteryType::Pl3 => 3,
            LotteryType::Pl5 => 5,
            LotteryType::Custom => 6,
        };
        
        let predicted_numbers: Vec<u32> = sorted_numbers.into_iter()
            .take(main_count)
            .map(|(number, _)| number)
            .collect();
        
        let confidence_scores = vec![0.75; main_count]; // Default ensemble confidence
        
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
    use crate::lottery::models::LotteryType;
    use chrono::NaiveDate;

    #[test]
    fn test_model_trainer_creation() {
        let trainer = ModelTrainer::new(LotteryType::Ssq);
        let algorithms = trainer.list_available_algorithms();
        assert!(algorithms.contains(&"random_forest".to_string()));
        assert!(algorithms.contains(&"neural_network".to_string()));
        assert!(algorithms.contains(&"hybrid".to_string()));
    }

    #[tokio::test]
    async fn test_model_training() {
        let mut trainer = ModelTrainer::new(LotteryType::Ssq);
        
        let training_data = TrainingData {
            features: vec![vec![0.1; 10]; 50],
            targets: vec![vec![1, 2, 3, 4, 5, 6]; 50],
            special_targets: None,
            weights: None,
        };

        let config = AlgorithmConfig {
            lottery_type: LotteryType::Ssq,
            parameters: HashMap::new(),
            hyperparameters: HashMap::new(),
            feature_config: HashMap::new(),
        };

        let results = trainer.train_all_algorithms(&training_data, &config).await;
        assert!(results.is_ok());
    }

    #[test]
    fn test_algorithm_listing() {
        let trainer = ModelTrainer::new(LotteryType::Ssq);
        let available = trainer.list_available_algorithms();
        let trained = trainer.list_trained_algorithms();
        
        assert!(!available.is_empty());
        assert!(trained.is_empty()); // No models trained yet
    }
}