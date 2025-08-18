use std::sync::Arc;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum LotteryError {
    #[error("数据库错误: {0}")]
    DatabaseError(#[from] sqlx::Error),
    
    #[error("序列化错误: {0}")]
    SerializationError(#[from] serde_json::Error),
    
    #[error("无效参数: {0}")]
    InvalidParameter(String),
    
    #[error("数据未找到: {0}")]
    NotFound(String),
    
    #[error("预测执行失败: {0}")]
    PredictionFailed(String),
    
    #[error("模型训练失败: {0}")]
    TrainingFailed(String),
    
    #[error("数据采集失败: {0}")]
    DataCollectionError(String),
    
    #[error("缓存错误: {0}")]
    CacheError(String),
    
    #[error("算法错误: {0}")]
    AlgorithmError(String),
    
    #[error("验证错误: {0}")]
    ValidationError(String),
    
    #[error("配置错误: {0}")]
    ConfigurationError(String),
    
    #[error("权限错误: {0}")]
    PermissionError(String),
    
    #[error("时间序列错误: {0}")]
    TimeSeriesError(String),
    
    #[error("特征工程错误: {0}")]
    FeatureEngineeringError(String),
    
    #[error("未知错误: {0}")]
    UnknownError(String),
}

pub type LotteryResult<T> = std::result::Result<T, LotteryError>;

#[derive(Error, Debug)]
pub enum DataValidationError {
    #[error("号码超出范围: {0}")]
    NumberOutOfRange(u32),
    
    #[error("号码数量不正确: 期望{expected}个，实际{actual}个")]
    InvalidNumberCount { expected: u32, actual: u32 },
    
    #[error("重复号码: {numbers:?}", numbers = .0)]
    DuplicateNumbers(Arc<Vec<u32>>),
    
    #[error("日期格式错误: {0}")]
    InvalidDateFormat(String),
    
    #[error("数据源格式错误: {0}")]
    InvalidDataSource(String),
    
    #[error("数据完整性错误: {0}")]
    DataIntegrityError(String),
}

#[derive(Error, Debug)]
pub enum ModelError {
    #[error("模型文件未找到: {0}")]
    ModelFileNotFound(String),
    
    #[error("模型格式错误: {0}")]
    ModelFormatError(String),
    
    #[error("模型版本不兼容: 期望{expected}，实际{actual}")]
    ModelVersionIncompatible { expected: String, actual: String },
    
    #[error("模型训练数据不足: 需要{required}条，实际{actual}条")]
    InsufficientTrainingData { required: u32, actual: u32 },
    
    #[error("模型过拟合: 训练准确率{train}，验证准确率{val}")]
    ModelOverfitting { train: f64, val: f64 },
    
    #[error("模型欠拟合: 验证准确率{val}低于阈值{threshold}")]
    ModelUnderfitting { val: f64, threshold: f64 },
    
    #[error("特征维度不匹配: 期望{expected}维，实际{actual}维")]
    FeatureDimensionMismatch { expected: u32, actual: u32 },
    
    #[error("内存不足: 需要{required}MB，可用{available}MB")]
    MemoryInsufficient { required: u64, available: u64 },
}

#[derive(Error, Debug)]
pub enum CacheError {
    #[error("缓存键不存在: {0}")]
    KeyNotFound(String),
    
    #[error("缓存过期: {0}")]
    CacheExpired(String),
    
    #[error("缓存连接失败: {0}")]
    ConnectionFailed(String),
    
    #[error("缓存空间不足: {0}")]
    CacheSpaceInsufficient(String),
    
    #[error("序列化失败: {0}")]
    SerializationFailed(String),
    
    #[error("反序列化失败: {0}")]
    DeserializationFailed(String),
}

