use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc, NaiveDate};
use std::collections::HashMap;


#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq, Eq, Hash)]
#[sqlx(type_name = "lottery_type", rename_all = "lowercase")]
pub enum LotteryType {
    Ssq,    // 双色球
    Dlt,    // 大乐透
    Fc3d,   // 福彩3D
    Pl3,    // 排列3
    Pl5,    // 排列5
    Custom,

}

impl std::fmt::Display for LotteryType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LotteryType::Ssq => write!(f, "ssq"),
            LotteryType::Dlt => write!(f, "dlt"),
            LotteryType::Fc3d => write!(f, "fc3d"),
            LotteryType::Pl3 => write!(f, "pl3"),
            LotteryType::Pl5 => write!(f, "pl5"),
            LotteryType::Custom => write!(f, "custom"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "algorithm_type", rename_all = "snake_case")]
pub enum AlgorithmType {
    RandomForest,
    Lstm,
    Arima,
    Statistical,
    NeuralNetwork,
    Hybrid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LotteryTypeConfig {
    pub id: Uuid,
    pub name: String,
    pub display_name: String,
    pub description: Option<String>,
    pub category: String,
    pub total_numbers: u32,
    pub special_numbers: u32,
    pub main_range_start: u32,
    pub main_range_end: u32,
    pub special_range_start: Option<u32>,
    pub special_range_end: Option<u32>,
    pub rules: serde_json::Value,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LotteryDrawing {
    pub id: Uuid,
    pub lottery_type: LotteryType,
    pub draw_number: String,
    pub draw_date: NaiveDate,
    pub draw_time: Option<String>,
    pub winning_numbers: Vec<u32>,
    pub special_numbers: Option<Vec<u32>>,
    pub jackpot_amount: Option<f64>,
    pub sales_amount: Option<f64>,
    pub prize_distribution: Option<serde_json::Value>,
    pub data_source: String,
    pub verification_status: String,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub crawled_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictionStrategy {
    pub id: Uuid,
    pub name: String,
    pub algorithm_type: AlgorithmType,
    pub description: Option<String>,
    pub parameters: serde_json::Value,
    pub hyperparameters: Option<serde_json::Value>,
    pub accuracy_rate: Option<f64>,
    pub precision_rate: Option<f64>,
    pub recall_rate: Option<f64>,
    pub f1_score: Option<f64>,
    pub total_predictions: u32,
    pub successful_predictions: u32,
    pub is_active: bool,
    pub is_public: bool,
    pub is_system: bool,
    pub owner_id: Option<Uuid>,
    pub version: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictionResult {
    pub id: Uuid,
    pub strategy_id: Uuid,
    pub lottery_type: LotteryType,
    pub predicted_numbers: Vec<u32>,
    pub predicted_special_numbers: Option<Vec<u32>>,
    pub confidence_scores: Vec<f64>,
    pub target_draw_date: NaiveDate,
    pub prediction_type: String,
    pub accuracy_score: Option<f64>,
    pub match_count: u32,
    pub special_match_count: u32,
    pub is_winner: bool,
    pub prize_tier: Option<u32>,
    pub prize_amount: Option<f64>,
    pub computation_time_ms: u32,
    pub feature_vector: Option<Vec<f64>>,
    pub metadata: Option<serde_json::Value>,
    pub prediction_date: DateTime<Utc>,
    pub validation_date: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisFeatures {
    pub id: Uuid,
    pub lottery_type: LotteryType,
    pub drawing_id: Option<Uuid>,
    pub feature_type: String,
    pub feature_name: String,
    pub feature_description: Option<String>,
    pub feature_data: serde_json::Value,
    pub feature_vector: Vec<f64>,
    pub data_points: u32,
    pub calculation_time_ms: u32,
    pub algorithm_version: String,
    pub is_valid: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelTrainingRecord {
    pub id: Uuid,
    pub strategy_id: Uuid,
    pub training_data_start: NaiveDate,
    pub training_data_end: NaiveDate,
    pub training_samples: u32,
    pub validation_samples: u32,
    pub test_samples: u32,
    pub model_parameters: serde_json::Value,
    pub training_accuracy: Option<f64>,
    pub validation_accuracy: Option<f64>,
    pub test_accuracy: Option<f64>,
    pub model_metrics: serde_json::Value,
    pub model_path: Option<String>,
    pub model_hash: Option<String>,
    pub model_size_bytes: Option<u64>,
    pub training_duration: Option<String>,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictionRequest {
    pub lottery_type: LotteryType,
    pub strategy_id: Uuid,
    pub count: u32,
    pub custom_parameters: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictionResponse {
    pub prediction_id: Uuid,
    pub numbers: Vec<u32>,
    pub special_numbers: Option<Vec<u32>>,
    pub confidence: f64,
    pub strategy_used: String,
    pub estimated_accuracy: f64,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrequencyAnalysis {
    pub lottery_type: LotteryType,
    pub numbers: Vec<(u32, u64)>,
    pub special_numbers: Option<Vec<(u32, u64)>>,
    pub period_start: NaiveDate,
    pub period_end: NaiveDate,
    pub total_draws: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendAnalysis {
    pub lottery_type: LotteryType,
    pub trends: Vec<TrendData>,
    pub correlations: HashMap<String, f64>,
    pub predictions: Vec<PredictiveInsight>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendData {
    pub metric: String,
    pub values: Vec<(NaiveDate, f64)>,
    pub trend: String,
    pub strength: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictiveInsight {
    pub insight_type: String,
    pub description: String,
    pub confidence: f64,
    pub recommendations: Vec<String>,
}