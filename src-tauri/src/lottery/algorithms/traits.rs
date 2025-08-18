
use crate::lottery::models::{LotteryDrawing, LotteryType};
use crate::lottery::errors::LotteryResult as Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Debug as DebugTrait;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictionInput {
    pub lottery_type: LotteryType,
    pub historical_data: Vec<LotteryDrawing>,
    pub target_date: chrono::NaiveDate,
    pub additional_features: Option<HashMap<String, f64>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictionOutput {
    pub predicted_numbers: Vec<u32>,
    pub predicted_special_numbers: Option<Vec<u32>>,
    pub confidence_scores: Vec<f64>,
    pub algorithm_metadata: HashMap<String, serde_json::Value>,
    pub computation_time_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingData {
    pub features: Vec<Vec<f64>>,
    pub targets: Vec<Vec<u32>>,
    pub special_targets: Option<Vec<Vec<u32>>>,
    pub weights: Option<Vec<f64>>,
}

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

#[async_trait]
pub trait PredictionAlgorithm: Send + Sync + DebugTrait {
    fn name(&self) -> String;
    
    fn algorithm_type(&self) -> String;
    
    async fn train(&mut self, training_data: &TrainingData, config: &AlgorithmConfig) -> Result<f64>;
    
    async fn predict(&self, input: &PredictionInput) -> Result<PredictionOutput>;
    
    async fn evaluate(&self, test_data: &TrainingData) -> Result<EvaluationMetrics>;
    
    fn is_trained(&self) -> bool;
    
    fn get_feature_importance(&self) -> Option<HashMap<String, f64>>;
    
    fn save_model(&self, path: &str) -> Result<()>;
    
    fn load_model(&mut self, path: &str) -> Result<()>;
    
    fn box_clone(&self) -> Box<dyn PredictionAlgorithm>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationMetrics {
    pub accuracy: f64,
    pub precision: f64,
    pub recall: f64,
    pub f1_score: f64,
    pub mean_absolute_error: f64,
    pub root_mean_squared_error: f64,
    pub confusion_matrix: Option<Vec<Vec<usize>>>,
    pub feature_importance: Option<HashMap<String, f64>>,
    pub cross_validation_scores: Option<Vec<f64>>,
}

impl Default for EvaluationMetrics {
    fn default() -> Self {
        Self {
            accuracy: 0.0,
            precision: 0.0,
            recall: 0.0,
            f1_score: 0.0,
            mean_absolute_error: 0.0,
            root_mean_squared_error: 0.0,
            confusion_matrix: None,
            feature_importance: None,
            cross_validation_scores: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureConfig {
    pub enable_frequency_analysis: bool,
    pub enable_trend_analysis: bool,
    pub enable_statistical_analysis: bool,
    pub enable_pattern_analysis: bool,
    pub enable_temporal_analysis: bool,
    pub window_size: usize,
    pub include_special_numbers: bool,
    pub feature_scaling: bool,
    pub dimensionality_reduction: Option<String>,
}

impl Default for FeatureConfig {
    fn default() -> Self {
        Self {
            enable_frequency_analysis: true,
            enable_trend_analysis: true,
            enable_statistical_analysis: true,
            enable_pattern_analysis: true,
            enable_temporal_analysis: true,
            window_size: 50,
            include_special_numbers: true,
            feature_scaling: true,
            dimensionality_reduction: None,
        }
    }
}

pub trait FeatureExtractor: Send + Sync {
    fn extract_features(&self, drawings: &[LotteryDrawing], config: &FeatureConfig) -> Result<TrainingData>;
    
    fn extract_single_features(&self, drawing: &LotteryDrawing, historical_data: &[LotteryDrawing]) -> Result<Vec<f64>>;
    
    fn get_feature_names(&self) -> Vec<String>;
    
    fn validate_features(&self, features: &[f64]) -> Result<bool>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelMetadata {
    pub algorithm_name: String,
    pub version: String,
    pub training_date: chrono::DateTime<chrono::Utc>,
    pub training_samples: usize,
    pub model_size_bytes: usize,
    pub hyperparameters: HashMap<String, serde_json::Value>,
    pub evaluation_metrics: EvaluationMetrics,
    pub lottery_type: LotteryType,
}

#[async_trait]
pub trait ModelPersistence: Send + Sync {
    async fn save(&self, path: &str, metadata: &ModelMetadata) -> Result<()>;
    async fn load(&self, path: &str) -> Result<(Box<dyn PredictionAlgorithm>, ModelMetadata)>;
    async fn list_models(&self, directory: &str) -> Result<Vec<ModelMetadata>>;
    async fn delete_model(&self, path: &str) -> Result<()>;
}
