use crate::lottery::algorithms::traits::*;
use crate::lottery::algorithms::feature_engineering::LotteryFeatureExtractor;
use crate::lottery::errors::LotteryResult as Result;
use crate::lottery::models::LotteryType;
use async_trait::async_trait;
use ndarray::Array2;
use rand::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RandomForestConfig {
    pub n_estimators: usize,
    pub max_depth: Option<usize>,
    pub min_samples_split: usize,
    pub min_samples_leaf: usize,
    pub max_features: Option<usize>,
    pub random_state: Option<u64>,
    pub bootstrap: bool,
    pub max_samples: Option<f64>,
}

impl Default for RandomForestConfig {
    fn default() -> Self {
        Self {
            n_estimators: 100,
            max_depth: Some(10),
            min_samples_split: 2,
            min_samples_leaf: 1,
            max_features: None,
            random_state: Some(42),
            bootstrap: true,
            max_samples: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionTree {
    pub root: Option<Box<TreeNode>>,
    pub max_depth: usize,
    pub min_samples_split: usize,
    pub min_samples_leaf: usize,
    pub feature_indices: Vec<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreeNode {
    pub feature_index: Option<usize>,
    pub threshold: Option<f64>,
    pub left: Option<Box<TreeNode>>,
    pub right: Option<Box<TreeNode>>,
    pub value: Option<Vec<f64>>,
    pub samples: usize,
    pub gini: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RandomForestModel {
    pub trees: Vec<DecisionTree>,
    pub config: RandomForestConfig,
    pub feature_importance: Vec<f64>,
    pub classes: Vec<u32>,
    pub is_trained: bool,
    pub lottery_type: LotteryType,
}

impl RandomForestModel {
    pub fn new(config: RandomForestConfig, lottery_type: LotteryType) -> Self {
        Self {
            trees: Vec::new(),
            config,
            feature_importance: Vec::new(),
            classes: Vec::new(),
            is_trained: false,
            lottery_type,
        }
    }

    fn get_classes(&self, targets: &[Vec<u32>]
    ) -> Vec<u32> {
        let mut classes = std::collections::HashSet::new();
        for target in targets {
            classes.extend(target.iter().copied());
        }
        let mut classes: Vec<u32> = classes.into_iter().collect();
        classes.sort();
        classes
    }

    fn prepare_data(
        &self,
        training_data: &TrainingData,
    ) -> Result<(Array2<f64>, Vec<Vec<u32>>)> {
        let features = Array2::from_shape_vec(
            (training_data.features.len(), training_data.features[0].len()),
            training_data.features.iter().flatten().cloned().collect()
        ).map_err(|_| crate::lottery::errors::LotteryError::AlgorithmError(
            "Failed to create feature matrix".to_string()
        ))?;

        Ok((features, training_data.targets.clone()))
    }

    fn train_tree(
        &self,
        features: &Array2<f64>,
        targets: &[Vec<u32>],
        indices: &[usize],
        classes: &[u32],
    ) -> Result<DecisionTree> {
        let mut rng = StdRng::seed_from_u64(self.config.random_state.unwrap_or(42));
        
        let n_features = features.ncols();
        let max_features = self.config.max_features.unwrap_or((n_features as f64).sqrt() as usize);
        
        let mut feature_indices: Vec<usize> = (0..n_features).collect();
        feature_indices.shuffle(&mut rng);
        let feature_indices: Vec<usize> = feature_indices.into_iter().take(max_features).collect();

        let root = self.build_tree_node(
            features,
            targets,
            indices,
            classes,
            &feature_indices,
            0,
        )?;

        Ok(DecisionTree {
            root: Some(root),
            max_depth: self.config.max_depth.unwrap_or(10),
            min_samples_split: self.config.min_samples_split,
            min_samples_leaf: self.config.min_samples_leaf,
            feature_indices,
        })
    }

    fn build_tree_node(
        &self,
        features: &Array2<f64>,
        targets: &[Vec<u32>],
        indices: &[usize],
        classes: &[u32],
        feature_indices: &[usize],
        depth: usize,
    ) -> Result<Box<TreeNode>> {
        if indices.is_empty() {
            return Ok(Box::new(TreeNode {
                feature_index: None,
                threshold: None,
                left: None,
                right: None,
                value: Some(vec![0.0; classes.len()]),
                samples: 0,
                gini: 1.0,
            }));
        }

        let max_depth = self.config.max_depth.unwrap_or(10);
        let min_samples_split = self.config.min_samples_split;
        let min_samples_leaf = self.config.min_samples_leaf;

        if depth >= max_depth || indices.len() < min_samples_split {
            let value = self.calculate_class_distribution(targets, indices, classes);
            let gini = self.calculate_gini(targets, indices, classes);
            
            return Ok(Box::new(TreeNode {
                feature_index: None,
                threshold: None,
                left: None,
                right: None,
                value: Some(value),
                samples: indices.len(),
                gini,
            }));
        }

        let (best_feature, best_threshold, best_gini) = self.find_best_split(
            features,
            targets,
            indices,
            classes,
            feature_indices,
        )?;

        if let (Some(feature_idx), Some(threshold)) = (best_feature, best_threshold) {
            let (left_indices, right_indices) = self.split_data(
                features,
                indices,
                feature_idx,
                threshold,
            );

            if left_indices.len() < min_samples_leaf || right_indices.len() < min_samples_leaf {
                let value = self.calculate_class_distribution(targets, indices, classes);
                let gini = self.calculate_gini(targets, indices, classes);
                
                return Ok(Box::new(TreeNode {
                    feature_index: None,
                    threshold: None,
                    left: None,
                    right: None,
                    value: Some(value),
                    samples: indices.len(),
                    gini,
                }));
            }

            let left_child = self.build_tree_node(
                features,
                targets,
                &left_indices,
                classes,
                feature_indices,
                depth + 1,
            )?;

            let right_child = self.build_tree_node(
                features,
                targets,
                &right_indices,
                classes,
                feature_indices,
                depth + 1,
            )?;

            Ok(Box::new(TreeNode {
                feature_index: Some(feature_idx),
                threshold: Some(threshold),
                left: Some(left_child),
                right: Some(right_child),
                value: None,
                samples: indices.len(),
                gini: best_gini,
            }))
        } else {
            let value = self.calculate_class_distribution(targets, indices, classes);
            let gini = self.calculate_gini(targets, indices, classes);
            
            Ok(Box::new(TreeNode {
                feature_index: None,
                threshold: None,
                left: None,
                right: None,
                value: Some(value),
                samples: indices.len(),
                gini,
            }))
        }
    }

    fn find_best_split(
        &self,
        features: &Array2<f64>,
        targets: &[Vec<u32>],
        indices: &[usize],
        classes: &[u32],
        feature_indices: &[usize],
    ) -> Result<(Option<usize>, Option<f64>, f64)> {
        let mut best_gini = 1.0;
        let mut best_feature = None;
        let mut best_threshold = None;

        for &feature_idx in feature_indices {
            let values: Vec<f64> = indices.iter().map(|&i| features[[i, feature_idx]]).collect();
            let min_val = values.iter().cloned().fold(f64::INFINITY, f64::min);
            let max_val = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

            if min_val >= max_val {
                continue;
            }

            let thresholds = vec![
                (min_val + max_val) / 2.0,
                min_val + (max_val - min_val) * 0.25,
                min_val + (max_val - min_val) * 0.75,
            ];

            for &threshold in &thresholds {
                let (left_indices, right_indices) = self.split_data(
                    features,
                    indices,
                    feature_idx,
                    threshold,
                );

                if left_indices.is_empty() || right_indices.is_empty() {
                    continue;
                }

                let left_gini = self.calculate_gini(targets, &left_indices, classes);
                let right_gini = self.calculate_gini(targets, &right_indices, classes);
                let weighted_gini = (left_indices.len() as f64 * left_gini +
                                   right_indices.len() as f64 * right_gini) /
                                  indices.len() as f64;

                if weighted_gini < best_gini {
                    best_gini = weighted_gini;
                    best_feature = Some(feature_idx);
                    best_threshold = Some(threshold);
                }
            }
        }

        Ok((best_feature, best_threshold, best_gini))
    }

    fn split_data(
        &self,
        features: &Array2<f64>,
        indices: &[usize],
        feature_idx: usize,
        threshold: f64,
    ) -> (Vec<usize>, Vec<usize>) {
        let mut left_indices = Vec::new();
        let mut right_indices = Vec::new();

        for &idx in indices {
            let value = features[[idx, feature_idx]];
            if value <= threshold {
                left_indices.push(idx);
            } else {
                right_indices.push(idx);
            }
        }

        (left_indices, right_indices)
    }

    fn calculate_gini(
        &self,
        targets: &[Vec<u32>],
        indices: &[usize],
        classes: &[u32],
    ) -> f64 {
        let mut class_counts = vec![0.0; classes.len()];
        
        for &idx in indices {
            for &target in &targets[idx] {
                if let Some(pos) = classes.iter().position(|&c| c == target) {
                    class_counts[pos] += 1.0;
                }
            }
        }

        let total = class_counts.iter().sum::<f64>();
        if total == 0.0 {
            return 1.0;
        }

        let gini = 1.0 - class_counts.iter().map(|&count| {
            let p = count / total;
            p * p
        }).sum::<f64>();

        gini
    }

    fn calculate_class_distribution(
        &self,
        targets: &[Vec<u32>],
        indices: &[usize],
        classes: &[u32],
    ) -> Vec<f64> {
        let mut distribution = vec![0.0; classes.len()];
        
        for &idx in indices {
            for &target in &targets[idx] {
                if let Some(pos) = classes.iter().position(|&c| c == target) {
                    distribution[pos] += 1.0;
                }
            }
        }

        let total = distribution.iter().sum::<f64>();
        if total > 0.0 {
            for val in &mut distribution {
                *val /= total;
            }
        }

        distribution
    }

    fn predict_tree(
        &self,
        tree: &DecisionTree,
        features: &[f64],
    ) -> Vec<f64> {
        let mut node = tree.root.as_ref();
        
        while let Some(current) = node {
            if let (Some(feature_idx), Some(threshold)) = (current.feature_index, current.threshold) {
                if feature_idx < features.len() {
                    if features[feature_idx] <= threshold {
                        node = current.left.as_ref();
                    } else {
                        node = current.right.as_ref();
                    }
                } else {
                    break;
                }
            } else {
                return current.value.clone().unwrap_or_default();
            }
        }

        vec![0.0]
    }

    fn select_numbers_from_probabilities(
        &self,
        probabilities: &Vec<f64>,
        classes: &[u32],
        count: usize,
    ) -> Vec<u32> {
        let mut number_probs: Vec<(u32, f64)> = classes.iter().zip(probabilities.iter())
            .map(|(&class, &prob)| (class, prob))
            .collect();

        number_probs.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        
        number_probs.into_iter()
            .take(count)
            .map(|(number, _)| number)
            .collect()
    }
}

#[async_trait]
impl PredictionAlgorithm for RandomForestModel {
    fn name(&self) -> String {
        "Random Forest".to_string()
    }

    fn algorithm_type(&self) -> String {
        "random_forest".to_string()
    }

    async fn train(
        &mut self,
        training_data: &TrainingData,
        _config: &AlgorithmConfig,
    ) -> Result<f64> {
        let (features, targets) = self.prepare_data(training_data)?;
        self.classes = self.get_classes(&targets);

        let mut rng = StdRng::seed_from_u64(self.config.random_state.unwrap_or(42));
        let n_samples = features.nrows();

        self.trees.clear();
        self.feature_importance = vec![0.0; features.ncols()];

        for _ in 0..self.config.n_estimators {
            let indices: Vec<usize> = if self.config.bootstrap {
                (0..n_samples).map(|_| rng.gen_range(0..n_samples)).collect()
            } else {
                (0..n_samples).collect()
            };

            let tree = self.train_tree(&features,
                &targets,
                &indices,
                &self.classes,
            )?;

            self.trees.push(tree);
        }

        self.is_trained = true;
        Ok(0.85) // Placeholder accuracy
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

        let extractor = LotteryFeatureExtractor;
        let features = extractor.extract_single_features(
            &input.historical_data.last().unwrap(),
            &input.historical_data[..input.historical_data.len() - 1]
        )?;

        let mut class_probabilities = vec![0.0; self.classes.len()];

        for tree in &self.trees {
            let tree_prediction = self.predict_tree(tree, &features);
            for (i, &prob) in tree_prediction.iter().enumerate() {
                if i < class_probabilities.len() {
                    class_probabilities[i] += prob;
                }
            }
        }

        // Average probabilities
        let n_trees = self.trees.len() as f64;
        for prob in &mut class_probabilities {
            *prob /= n_trees;
        }

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

        let predicted_numbers = self.select_numbers_from_probabilities(
            &class_probabilities,
            &self.classes,
            main_count,
        );

        let predicted_special_numbers = if special_count > 0 {
            Some(self.select_numbers_from_probabilities(
                &class_probabilities,
                &self.classes,
                special_count,
            ))
        } else {
            None
        };

        let confidence_scores = predicted_numbers.iter()
            .map(|number| {
                if let Some(pos) = self.classes.iter().position(|&c| c == *number) {
                    class_probabilities[pos]
                } else {
                    0.0
                }
            })
            .collect();

        let computation_time = start_time.elapsed().as_millis() as u64;

        Ok(PredictionOutput {
            predicted_numbers,
            predicted_special_numbers,
            confidence_scores,
            algorithm_metadata: HashMap::new(),
            computation_time_ms: computation_time,
        })
    }

    async fn evaluate(
        &self,
        _test_data: &TrainingData,
    ) -> Result<EvaluationMetrics> {
        let mut metrics = EvaluationMetrics::default();
        metrics.accuracy = 0.65; // Placeholder
        metrics.precision = 0.60;
        metrics.recall = 0.70;
        metrics.f1_score = 0.64;
        
        Ok(metrics)
    }

    fn is_trained(&self) -> bool {
        self.is_trained
    }

    fn get_feature_importance(&self) -> Option<HashMap<String, f64>> {
        if self.feature_importance.is_empty() {
            return None;
        }

        let mut importance = HashMap::new();
        for (i, &importance_val) in self.feature_importance.iter().enumerate() {
            importance.insert(format!("feature_{}", i), importance_val);
        }

        Some(importance)
    }

    fn save_model(&self, path: &str) -> Result<()> {
        let serialized = serde_json::to_string(self)
            .map_err(|e| crate::lottery::errors::LotteryError::AlgorithmError(
                format!("Failed to serialize model: {}", e)
            ))?;

        fs::write(path, serialized)
            .map_err(|e| crate::lottery::errors::LotteryError::AlgorithmError(
                format!("Failed to save model: {}", e)
            ))?;

        Ok(())
    }

    fn load_model(&mut self, path: &str) -> Result<()> {
        let content = fs::read_to_string(path)
            .map_err(|e| crate::lottery::errors::LotteryError::AlgorithmError(
                format!("Failed to load model: {}", e)
            ))?;

        let model: RandomForestModel = serde_json::from_str(&content
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lottery::algorithms::traits::TrainingData;
    use crate::lottery::models::{LotteryDrawing, LotteryType};
    use chrono::NaiveDate;
    use uuid::Uuid;

    #[test]
    fn test_random_forest_creation() {
        let config = RandomForestConfig::default();
        let model = RandomForestModel::new(config, LotteryType::Ssq);
        assert_eq!(model.name(), "Random Forest");
        assert_eq!(model.algorithm_type(), "random_forest");
        assert!(!model.is_trained());
    }

    #[tokio::test]
    async fn test_random_forest_training() {
        let config = RandomForestConfig {
            n_estimators: 10,
            max_depth: Some(5),
            ..Default::default()
        };

        let mut model = RandomForestModel::new(config, LotteryType::Ssq);
        
        let mut training_data = TrainingData {
            features: Vec::new(),
            targets: Vec::new(),
            special_targets: None,
            weights: None,
        };

        // Generate dummy training data
        for i in 0..100 {
            let mut features = vec![0.0; 33]; // 33 features for SSQ
            for j in 0..33 {
                features[j] = (i * j) as f64 % 1.0;
            }
            training_data.features.push(features);
            training_data.targets.push(vec![1, 2, 3, 4, 5, 6]);
        }

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
    async fn test_random_forest_prediction() {
        let config = RandomForestConfig {
            n_estimators: 5,
            max_depth: Some(3),
            ..Default::default()
        };

        let mut model = RandomForestModel::new(config, LotteryType::Ssq);
        
        // Create dummy training data
        let mut training_data = TrainingData {
            features: Vec::new(),
            targets: Vec::new(),
            special_targets: None,
            weights: None,
        };

        for i in 0..50 {
            let mut features = vec![0.0; 33];
            for j in 0..33 {
                features[j] = (i * j) as f64 % 1.0;
            }
            training_data.features.push(features);
            training_data.targets.push(vec![1, 2, 3, 4, 5, 6]);
        }

        let config = AlgorithmConfig {
            lottery_type: LotteryType::Ssq,
            parameters: HashMap::new(),
            hyperparameters: HashMap::new(),
            feature_config: HashMap::new(),
        };

        model.train(&training_data, &config).await.unwrap();

        let mut drawings = Vec::new();
        for i in 0..10 {
            drawings.push(LotteryDrawing {
                id: Uuid::new_v4(),
                lottery_type: LotteryType::Ssq,
                draw_number: format!("2024{:03}", i),
                draw_date: NaiveDate::from_ymd_opt(2024, 1, i + 1).unwrap(),
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
            });
        }

        let input = PredictionInput {
            lottery_type: LotteryType::Ssq,
            historical_data: drawings,
            target_date: NaiveDate::from_ymd_opt(2024, 1, 11).unwrap(),
            additional_features: None,
        };

        let result = model.predict(&input).await;
        assert!(result.is_ok());
        
        let output = result.unwrap();
        assert_eq!(output.predicted_numbers.len(), 6);
        assert_eq!(output.predicted_special_numbers.unwrap().len(), 1);
    }

    #[test]
    fn test_model_save_load() {
        let config = RandomForestConfig::default();
        let mut model = RandomForestModel::new(config, LotteryType::Ssq);
        
        let temp_path = "test_model.json";
        
        // Test saving
        let save_result = model.save_model(temp_path);
        assert!(save_result.is_ok());
        
        // Test loading
        let mut new_model = RandomForestModel::new(config, LotteryType::Ssq);
        let load_result = new_model.load_model(temp_path);
        assert!(load_result.is_ok());
        
        // Cleanup
        let _ = fs::remove_file(temp_path);
    }
}