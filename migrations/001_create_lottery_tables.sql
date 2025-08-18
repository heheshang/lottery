-- 彩票预测系统数据库迁移
-- 创建彩票类型表
CREATE TABLE lottery_types (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(50) NOT NULL UNIQUE,
    display_name VARCHAR(100) NOT NULL,
    description TEXT,
    category VARCHAR(20) NOT NULL CHECK (category IN ('welfare', 'sports', 'local')),
    total_numbers INTEGER NOT NULL CHECK (total_numbers > 0),
    special_numbers INTEGER CHECK (special_numbers >= 0),
    main_range_start INTEGER NOT NULL DEFAULT 1,
    main_range_end INTEGER NOT NULL,
    special_range_start INTEGER,
    special_range_end INTEGER,
    rules JSONB NOT NULL DEFAULT '{}',
    is_active BOOLEAN DEFAULT TRUE,
    is_deprecated BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    CHECK (main_range_end > main_range_start),
    CHECK (special_range_end IS NULL OR special_range_end > special_range_start)
);

-- 创建历史开奖数据表
CREATE TABLE lottery_drawings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    lottery_type_id UUID NOT NULL REFERENCES lottery_types(id),
    draw_number VARCHAR(20) NOT NULL,
    draw_date DATE NOT NULL,
    draw_time TIME,
    winning_numbers INTEGER[] NOT NULL,
    special_numbers INTEGER[],
    jackpot_amount DECIMAL(15,2),
    sales_amount DECIMAL(15,2),
    prize_pool_amount DECIMAL(15,2),
    prize_distribution JSONB DEFAULT '{}',
    data_source VARCHAR(50) NOT NULL,
    source_url VARCHAR(500),
    verification_hash VARCHAR(64),
    verification_status VARCHAR(20) DEFAULT 'pending' CHECK (
        verification_status IN ('pending', 'verified', 'failed', 'duplicate')
    ),
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    crawled_at TIMESTAMP WITH TIME ZONE,
    UNIQUE(lottery_type_id, draw_number),
    UNIQUE(lottery_type_id, draw_date)
);

-- 创建预测策略表
CREATE TABLE prediction_strategies (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL,
    algorithm_type VARCHAR(50) NOT NULL CHECK (
        algorithm_type IN ('random_forest', 'lstm', 'arima', 'statistical', 'neural_network', 'hybrid')
    ),
    description TEXT,
    parameters JSONB NOT NULL DEFAULT '{}',
    hyperparameters JSONB DEFAULT '{}',
    feature_config JSONB DEFAULT '{}',
    accuracy_rate DECIMAL(5,2) CHECK (accuracy_rate >= 0 AND accuracy_rate <= 100),
    precision_rate DECIMAL(5,2) CHECK (precision_rate >= 0 AND precision_rate <= 100),
    recall_rate DECIMAL(5,2) CHECK (recall_rate >= 0 AND recall_rate <= 100),
    f1_score DECIMAL(5,2) CHECK (f1_score >= 0 AND f1_score <= 100),
    total_predictions INTEGER DEFAULT 0,
    successful_predictions INTEGER DEFAULT 0,
    total_trainings INTEGER DEFAULT 0,
    last_training_date TIMESTAMP WITH TIME ZONE,
    model_path VARCHAR(500),
    model_hash VARCHAR(64),
    model_size_bytes BIGINT,
    is_active BOOLEAN DEFAULT TRUE,
    is_public BOOLEAN DEFAULT FALSE,
    is_system BOOLEAN DEFAULT FALSE,
    owner_id UUID,
    version VARCHAR(20) DEFAULT '1.0.0',
    parent_strategy_id UUID REFERENCES prediction_strategies(id),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- 创建预测结果表
CREATE TABLE prediction_results (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    strategy_id UUID NOT NULL REFERENCES prediction_strategies(id),
    lottery_type_id UUID NOT NULL REFERENCES lottery_types(id),
    actual_draw_id UUID REFERENCES lottery_drawings(id),
    predicted_numbers INTEGER[] NOT NULL,
    predicted_special_numbers INTEGER[],
    confidence_scores DECIMAL(3,2)[] CHECK (
        ARRAY_LENGTH(confidence_scores, 1) = 
        ARRAY_LENGTH(predicted_numbers, 1) + 
        COALESCE(ARRAY_LENGTH(predicted_special_numbers, 1), 0)
    ),
    target_draw_date DATE NOT NULL,
    prediction_type VARCHAR(20) DEFAULT 'standard' CHECK (
        prediction_type IN ('standard', 'quick', 'detailed', 'batch')
    ),
    accuracy_score DECIMAL(5,2),
    match_count INTEGER DEFAULT 0,
    special_match_count INTEGER DEFAULT 0,
    is_winner BOOLEAN DEFAULT FALSE,
    prize_tier INTEGER,
    prize_amount DECIMAL(15,2),
    computation_time_ms INTEGER,
    feature_vector FLOAT[],
    metadata JSONB DEFAULT '{}',
    prediction_date TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    validation_date TIMESTAMP WITH TIME ZONE,
    INDEX idx_strategy_target (strategy_id, target_draw_date),
    INDEX idx_lottery_prediction (lottery_type_id, prediction_date DESC),
    INDEX idx_winner_results (is_winner, prize_amount DESC)
);

-- 创建分析特征表
CREATE TABLE analysis_features (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    lottery_type_id UUID NOT NULL REFERENCES lottery_types(id),
    drawing_id UUID REFERENCES lottery_drawings(id),
    feature_type VARCHAR(50) NOT NULL CHECK (
        feature_type IN ('frequency', 'trend', 'statistical', 'pattern', 'temporal')
    ),
    feature_name VARCHAR(100) NOT NULL,
    feature_description TEXT,
    feature_data JSONB NOT NULL,
    feature_vector FLOAT[] NOT NULL,
    feature_hash VARCHAR(64),
    data_points INTEGER,
    calculation_time_ms INTEGER,
    algorithm_version VARCHAR(20) DEFAULT '1.0.0',
    is_valid BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    INDEX idx_lottery_feature (lottery_type_id, feature_type),
    INDEX idx_drawing_feature (drawing_id),
    INDEX idx_feature_hash (feature_hash)
);

-- 创建模型训练记录表
CREATE TABLE model_training_records (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    strategy_id UUID NOT NULL REFERENCES prediction_strategies(id),
    training_data_start DATE NOT NULL,
    training_data_end DATE NOT NULL,
    training_samples INTEGER NOT NULL,
    validation_samples INTEGER NOT NULL,
    test_samples INTEGER NOT NULL,
    model_parameters JSONB NOT NULL,
    feature_config JSONB NOT NULL,
    training_accuracy DECIMAL(5,2),
    validation_accuracy DECIMAL(5,2),
    test_accuracy DECIMAL(5,2),
    training_loss DECIMAL(8,4),
    validation_loss DECIMAL(8,4),
    model_metrics JSONB NOT NULL,
    confusion_matrix JSONB,
    feature_importance JSONB,
    model_path VARCHAR(500),
    model_hash VARCHAR(64),
    model_size_bytes BIGINT,
    training_duration INTERVAL,
    hardware_info JSONB DEFAULT '{}',
    status VARCHAR(20) DEFAULT 'pending' CHECK (
        status IN ('pending', 'running', 'completed', 'failed', 'cancelled')
    ),
    error_message TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    started_at TIMESTAMP WITH TIME ZONE,
    completed_at TIMESTAMP WITH TIME ZONE,
    INDEX idx_strategy_training (strategy_id, created_at DESC),
    INDEX idx_training_status (status)
);

-- 创建用户策略配置表
CREATE TABLE user_strategy_configs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL,
    strategy_id UUID NOT NULL REFERENCES prediction_strategies(id),
    custom_parameters JSONB NOT NULL DEFAULT '{}',
    custom_filters JSONB DEFAULT '{}',
    notification_enabled BOOLEAN DEFAULT TRUE,
    auto_predict BOOLEAN DEFAULT FALSE,
    usage_count INTEGER DEFAULT 0,
    success_count INTEGER DEFAULT 0,
    favorite_count INTEGER DEFAULT 0,
    is_favorite BOOLEAN DEFAULT FALSE,
    is_active BOOLEAN DEFAULT TRUE,
    last_used_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(user_id, strategy_id),
    INDEX idx_user_strategies (user_id, is_favorite DESC)
);

-- 创建缓存数据表
CREATE TABLE cache_data (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    cache_key VARCHAR(255) NOT NULL UNIQUE,
    cache_type VARCHAR(50) NOT NULL,
    data_content JSONB NOT NULL,
    data_hash VARCHAR(64),
    ttl_seconds INTEGER DEFAULT 3600,
    priority INTEGER DEFAULT 0,
    access_count INTEGER DEFAULT 0,
    hit_count INTEGER DEFAULT 0,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    expires_at TIMESTAMP WITH TIME ZONE,
    last_accessed TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    INDEX idx_cache_type (cache_type),
    INDEX idx_cache_expires (expires_at),
    INDEX idx_cache_priority (priority DESC)
);

-- 创建性能优化索引
CREATE INDEX idx_drawings_lottery_date ON lottery_drawings(lottery_type_id, draw_date DESC);
CREATE INDEX idx_drawings_number_gin ON lottery_drawings USING GIN(winning_numbers);
CREATE INDEX idx_drawings_special_gin ON lottery_drawings USING GIN(special_numbers);
CREATE INDEX idx_predictions_strategy_target ON prediction_results(strategy_id, target_draw_date);
CREATE INDEX idx_predictions_lottery_date ON prediction_results(lottery_type_id, prediction_date DESC);
CREATE INDEX idx_predictions_winner ON prediction_results(is_winner, prize_amount DESC) WHERE is_winner = true;
CREATE INDEX idx_features_lottery_drawing ON analysis_features(lottery_type_id, drawing_id);
CREATE INDEX idx_features_type_name ON analysis_features(feature_type, feature_name);
CREATE INDEX idx_strategy_performance ON prediction_strategies(algorithm_type, accuracy_rate DESC);

-- 创建全文搜索索引
CREATE INDEX idx_drawings_search ON lottery_drawings 
    USING gin(to_tsvector('english', draw_number || ' ' || COALESCE(metadata->>'notes', '')));