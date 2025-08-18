# 彩票预测系统数据库设计文档

> **文档版本**: v1.0.0  
> **创建日期**: 2024-08-18  
> **作者**: AI助手  
> **状态**: 设计阶段

## 📋 目录

1. [数据库架构](#数据库架构)
2. [数据模型设计](#数据模型设计)
3. [表结构详解](#表结构详解)
4. [索引策略](#索引策略)
5. [数据迁移方案](#数据迁移方案)
6. [备份恢复策略](#备份恢复策略)
7. [性能优化](#性能优化)
8. [监控运维](#监控运维)

---

## 数据库架构

### 1.1 整体架构

```
┌─────────────────────────────────────────┐
│           多层存储架构                   │
├─────────────────────────────────────────┤
│  应用层    │  缓存层    │  持久化层      │
│  DuckDB    │  Redis    │  PostgreSQL   │
│  (本地)    │  (内存)   │  (云端)       │
└─────────────────────────────────────────┘
           │           │           │
┌─────────────────────────────────────────┐
│           数据分层                       │
│  热数据  │  温数据  │  冷数据  │  归档数据 │
│  0-7天   │  7-30天  │  30-90天 │  90天+   │
└─────────────────────────────────────────┘
```

### 1.2 技术选型

| 数据库类型 | 选型 | 版本 | 使用场景 | 特点 |
|---|---|---|---|---|
| **关系型** | PostgreSQL | 15+ | 云端主存储 | ACID、复杂查询 |
| **嵌入式** | DuckDB | latest | 本地存储 | 高性能、零配置 |
| **缓存** | Redis | 7.x | 缓存层 | 内存存储、高速 |
| **时序** | TimescaleDB | 2.x | 扩展方案 | 时间序列优化 |

---

## 数据模型设计

### 2.1 概念模型 (ER图)

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   LotteryType   │    │  LotteryDrawing │    │  PredictionResult│
├─────────────────┤    ├─────────────────┤    ├─────────────────┤
│ PK id (UUID)    │────│ PK id (UUID)    │────│ PK id (UUID)    │
│     name        │    │ FK lottery_type │    │ FK strategy_id  │
│     rules       │    │     draw_number │    │ FK lottery_type │
│     ranges      │    │     draw_date   │    │     numbers     │
│     created_at  │    │     numbers[]   │    │     confidence  │
└─────────────────┘    │     jackpot     │    │     accuracy    │
                       │     created_at  │    │     created_at  │
                       └─────────────────┘    └─────────────────┘
                                │                       │
                       ┌─────────────────┐    ┌─────────────────┐
                       │   AnalysisFeatures     │  PredictionStrategy  │
                       ├─────────────────┤    ├─────────────────┤
                       │ PK id (UUID)    │    │ PK id (UUID)    │
                       │ FK lottery_type │    │     name        │
                       │ FK drawing_id   │    │     algorithm   │
                       │     features    │    │     parameters  │
                       │     vector[]    │    │     accuracy    │
                       └─────────────────┘    └─────────────────┘
```

### 2.2 数据字典

#### 2.2.1 主数据表

| 表名 | 描述 | 数据量估计 | 更新频率 |
|---|---|---|---|
| `lottery_types` | 彩票类型配置 | ~10条 | 极少 |
| `lottery_drawings` | 历史开奖数据 | ~50万条/年 | 每日 |
| `prediction_strategies` | 预测策略配置 | ~1000条 | 中等 |
| `prediction_results` | 预测结果记录 | ~100万条/年 | 高频 |
| `analysis_features` | 分析特征数据 | ~500万条/年 | 高频 |

---

## 表结构详解

### 3.1 彩票类型表 (lottery_types)

```sql
CREATE TABLE lottery_types (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    
    -- 基本信息
    name VARCHAR(50) NOT NULL UNIQUE,
    display_name VARCHAR(100) NOT NULL,
    description TEXT,
    category VARCHAR(20) NOT NULL CHECK (category IN ('welfare', 'sports', 'local')),
    
    -- 号码规则
    total_numbers INTEGER NOT NULL CHECK (total_numbers > 0),
    special_numbers INTEGER CHECK (special_numbers >= 0),
    
    -- 号码范围
    main_range_start INTEGER NOT NULL DEFAULT 1,
    main_range_end INTEGER NOT NULL,
    special_range_start INTEGER,
    special_range_end INTEGER,
    
    -- 游戏规则 (JSON格式)
    rules JSONB NOT NULL DEFAULT '{
        "selection_rules": {
            "main_numbers": {"min": 6, "max": 6},
            "special_numbers": {"min": 1, "max": 1}
        },
        "prize_rules": {
            "tiers": 6,
            "distribution": "pari-mutuel"
        }
    }',
    
    -- 状态管理
    is_active BOOLEAN DEFAULT TRUE,
    is_deprecated BOOLEAN DEFAULT FALSE,
    
    -- 时间戳
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    
    -- 约束
    CHECK (main_range_end > main_range_start),
    CHECK (special_range_end IS NULL OR special_range_end > special_range_start)
);

-- 初始数据
INSERT INTO lottery_types (name, display_name, category, total_numbers, special_numbers, 
                          main_range_end, special_range_end, rules) VALUES
('ssq', '双色球', 'welfare', 6, 1, 33, 16, 
 '{"selection_rules":{"main_numbers":{"min":6,"max":6},"special_numbers":{"min":1,"max":1}},"prize_rules":{"tiers":6}}'),
('dlt', '大乐透', 'sports', 5, 2, 35, 12,
 '{"selection_rules":{"main_numbers":{"min":5,"max":5},"special_numbers":{"min":2,"max":2}},"prize_rules":{"tiers":9}}'),
('fc3d', '福彩3D', 'welfare', 3, 0, 9, 9,
 '{"selection_rules":{"main_numbers":{"min":3,"max":3},"special_numbers":{"min":0,"max":0}},"prize_rules":{"tiers":3}}');
```

### 3.2 历史开奖数据表 (lottery_drawings)

```sql
CREATE TABLE lottery_drawings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    
    -- 关联关系
    lottery_type_id UUID NOT NULL REFERENCES lottery_types(id),
    
    -- 开奖信息
    draw_number VARCHAR(20) NOT NULL,
    draw_date DATE NOT NULL,
    draw_time TIME,
    
    -- 开奖号码
    winning_numbers INTEGER[] NOT NULL,
    special_numbers INTEGER[],
    
    -- 奖金信息
    jackpot_amount DECIMAL(15,2),
    sales_amount DECIMAL(15,2),
    prize_pool_amount DECIMAL(15,2),
    prize_distribution JSONB DEFAULT '{}',
    
    -- 数据来源和验证
    data_source VARCHAR(50) NOT NULL,
    source_url VARCHAR(500),
    verification_hash VARCHAR(64),
    verification_status VARCHAR(20) DEFAULT 'pending' CHECK (
        verification_status IN ('pending', 'verified', 'failed', 'duplicate')
    ),
    
    -- 元数据
    metadata JSONB DEFAULT '{}',
    
    -- 时间戳
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    crawled_at TIMESTAMP WITH TIME ZONE,
    
    -- 约束
    UNIQUE(lottery_type_id, draw_number),
    UNIQUE(lottery_type_id, draw_date)
);

-- 分区表设计（按年份分区）
CREATE TABLE lottery_drawings_2024 PARTITION OF lottery_drawings
    FOR VALUES FROM ('2024-01-01') TO ('2025-01-01');

CREATE TABLE lottery_drawings_2023 PARTITION OF lottery_drawings
    FOR VALUES FROM ('2023-01-01') TO ('2024-01-01');
```

### 3.3 预测策略表 (prediction_strategies)

```sql
CREATE TABLE prediction_strategies (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    
    -- 基本信息
    name VARCHAR(100) NOT NULL,
    description TEXT,
    algorithm_type VARCHAR(50) NOT NULL CHECK (
        algorithm_type IN ('random_forest', 'lstm', 'arima', 'statistical', 'neural_network', 'hybrid')
    ),
    
    -- 算法参数
    parameters JSONB NOT NULL DEFAULT '{}',
    hyperparameters JSONB DEFAULT '{}',
    feature_config JSONB DEFAULT '{}',
    
    -- 性能指标
    accuracy_rate DECIMAL(5,2) CHECK (accuracy_rate >= 0 AND accuracy_rate <= 100),
    precision_rate DECIMAL(5,2) CHECK (precision_rate >= 0 AND precision_rate <= 100),
    recall_rate DECIMAL(5,2) CHECK (recall_rate >= 0 AND recall_rate <= 100),
    f1_score DECIMAL(5,2) CHECK (f1_score >= 0 AND f1_score <= 100),
    
    -- 统计信息
    total_predictions INTEGER DEFAULT 0,
    successful_predictions INTEGER DEFAULT 0,
    total_trainings INTEGER DEFAULT 0,
    last_training_date TIMESTAMP WITH TIME ZONE,
    
    -- 模型文件
    model_path VARCHAR(500),
    model_hash VARCHAR(64),
    model_size BIGINT,
    
    -- 状态管理
    is_active BOOLEAN DEFAULT TRUE,
    is_public BOOLEAN DEFAULT FALSE,
    is_system BOOLEAN DEFAULT FALSE,
    
    -- 所有权
    owner_id UUID,
    
    -- 版本控制
    version VARCHAR(20) DEFAULT '1.0.0',
    parent_strategy_id UUID REFERENCES prediction_strategies(id),
    
    -- 时间戳
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    
    -- 约束
    CHECK (accuracy_rate IS NULL OR (accuracy_rate >= 0 AND accuracy_rate <= 100))
);

-- 常用策略模板
INSERT INTO prediction_strategies (name, algorithm_type, description, parameters, is_system) VALUES
('随机森林-基础版', 'random_forest', '基于随机森林的基础预测策略', 
 '{"n_estimators": 100, "max_depth": 10, "min_samples_split": 2}', true),
('LSTM-时间序列', 'lstm', '基于LSTM神经网络的时间序列预测', 
 '{"hidden_size": 128, "num_layers": 2, "dropout": 0.2, "epochs": 100}', true),
('统计分析-频率法', 'statistical', '基于统计频率的简单预测方法', 
 '{"window_size": 50, "weight_function": "linear"}', true);
```

### 3.4 预测结果表 (prediction_results)

```sql
CREATE TABLE prediction_results (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    
    -- 关联关系
    strategy_id UUID NOT NULL REFERENCES prediction_strategies(id),
    lottery_type_id UUID NOT NULL REFERENCES lottery_types(id),
    actual_draw_id UUID REFERENCES lottery_drawings(id),
    
    -- 预测内容
    predicted_numbers INTEGER[] NOT NULL,
    predicted_special_numbers INTEGER[],
    confidence_scores DECIMAL(3,2)[] CHECK (
        ARRAY_LENGTH(confidence_scores, 1) = 
        ARRAY_LENGTH(predicted_numbers, 1) + 
        COALESCE(ARRAY_LENGTH(predicted_special_numbers, 1), 0)
    ),
    
    -- 预测目标
    target_draw_date DATE NOT NULL,
    prediction_type VARCHAR(20) DEFAULT 'standard' CHECK (
        prediction_type IN ('standard', 'quick', 'detailed', 'batch')
    ),
    
    -- 验证结果
    accuracy_score DECIMAL(5,2),
    match_count INTEGER DEFAULT 0,
    special_match_count INTEGER DEFAULT 0,
    is_winner BOOLEAN DEFAULT FALSE,
    prize_tier INTEGER,
    prize_amount DECIMAL(15,2),
    
    -- 计算信息
    computation_time_ms INTEGER,
    feature_vector FLOAT[],
    
    -- 元数据
    metadata JSONB DEFAULT '{}',
    
    -- 时间戳
    prediction_date TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    validation_date TIMESTAMP WITH TIME ZONE,
    
    -- 索引
    INDEX idx_strategy_target (strategy_id, target_draw_date),
    INDEX idx_lottery_prediction (lottery_type_id, prediction_date DESC),
    INDEX idx_winner_results (is_winner, prize_amount DESC)
);

-- 分区表设计（按月分区）
CREATE TABLE prediction_results_202408 PARTITION OF prediction_results
    FOR VALUES FROM ('2024-08-01') TO ('2024-09-01');
```

### 3.5 分析特征表 (analysis_features)

```sql
CREATE TABLE analysis_features (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    
    -- 关联关系
    lottery_type_id UUID NOT NULL REFERENCES lottery_types(id),
    drawing_id UUID REFERENCES lottery_drawings(id),
    
    -- 特征信息
    feature_type VARCHAR(50) NOT NULL CHECK (
        feature_type IN ('frequency', 'trend', 'statistical', 'pattern', 'temporal')
    ),
    feature_name VARCHAR(100) NOT NULL,
    feature_description TEXT,
    
    -- 特征数据
    feature_data JSONB NOT NULL,
    feature_vector FLOAT[] NOT NULL,
    feature_hash VARCHAR(64),
    
    -- 特征统计
    data_points INTEGER,
    calculation_time_ms INTEGER,
    
    -- 版本控制
    algorithm_version VARCHAR(20) DEFAULT '1.0.0',
    is_valid BOOLEAN DEFAULT TRUE,
    
    -- 时间戳
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    
    -- 索引
    INDEX idx_lottery_feature (lottery_type_id, feature_type),
    INDEX idx_drawing_feature (drawing_id),
    INDEX idx_feature_hash (feature_hash)
);

-- 常用特征索引
CREATE INDEX idx_features_composite ON analysis_features(
    lottery_type_id, feature_type, feature_name, created_at DESC
);
```

### 3.6 模型训练记录表 (model_training_records)

```sql
CREATE TABLE model_training_records (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    
    -- 关联关系
    strategy_id UUID NOT NULL REFERENCES prediction_strategies(id),
    
    -- 训练数据信息
    training_data_start DATE NOT NULL,
    training_data_end DATE NOT NULL,
    training_samples INTEGER NOT NULL,
    validation_samples INTEGER NOT NULL,
    test_samples INTEGER NOT NULL,
    
    -- 模型参数
    model_parameters JSONB NOT NULL,
    feature_config JSONB NOT NULL,
    
    -- 性能指标
    training_accuracy DECIMAL(5,2),
    validation_accuracy DECIMAL(5,2),
    test_accuracy DECIMAL(5,2),
    training_loss DECIMAL(8,4),
    validation_loss DECIMAL(8,4),
    
    -- 详细指标
    model_metrics JSONB NOT NULL,
    confusion_matrix JSONB,
    feature_importance JSONB,
    
    -- 训练信息
    model_path VARCHAR(500),
    model_hash VARCHAR(64),
    model_size_bytes BIGINT,
    training_duration INTERVAL,
    
    -- 硬件信息
    hardware_info JSONB DEFAULT '{}',
    
    -- 状态管理
    status VARCHAR(20) DEFAULT 'pending' CHECK (
        status IN ('pending', 'running', 'completed', 'failed', 'cancelled')
    ),
    error_message TEXT,
    
    -- 时间戳
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    started_at TIMESTAMP WITH TIME ZONE,
    completed_at TIMESTAMP WITH TIME ZONE,
    
    -- 索引
    INDEX idx_strategy_training (strategy_id, created_at DESC),
    INDEX idx_training_status (status)
);
```

### 3.7 用户策略配置表 (user_strategy_configs)

```sql
CREATE TABLE user_strategy_configs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    
    -- 关联关系
    user_id UUID NOT NULL,
    strategy_id UUID NOT NULL REFERENCES prediction_strategies(id),
    
    -- 自定义配置
    custom_parameters JSONB NOT NULL DEFAULT '{}',
    custom_filters JSONB DEFAULT '{}',
    
    -- 偏好设置
    notification_enabled BOOLEAN DEFAULT TRUE,
    auto_predict BOOLEAN DEFAULT FALSE,
    
    -- 使用统计
    usage_count INTEGER DEFAULT 0,
    success_count INTEGER DEFAULT 0,
    favorite_count INTEGER DEFAULT 0,
    
    -- 状态管理
    is_favorite BOOLEAN DEFAULT FALSE,
    is_active BOOLEAN DEFAULT TRUE,
    
    -- 时间戳
    last_used_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    
    -- 约束
    UNIQUE(user_id, strategy_id),
    INDEX idx_user_strategies (user_id, is_favorite DESC)
);
```

### 3.8 缓存数据表 (cache_data)

```sql
CREATE TABLE cache_data (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    
    -- 缓存标识
    cache_key VARCHAR(255) NOT NULL UNIQUE,
    cache_type VARCHAR(50) NOT NULL,
    
    -- 缓存内容
    data_content JSONB NOT NULL,
    data_hash VARCHAR(64),
    
    -- 缓存策略
    ttl_seconds INTEGER DEFAULT 3600,
    priority INTEGER DEFAULT 0,
    
    -- 统计信息
    access_count INTEGER DEFAULT 0,
    hit_count INTEGER DEFAULT 0,
    
    -- 时间戳
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    expires_at TIMESTAMP WITH TIME ZONE,
    last_accessed TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    
    -- 索引
    INDEX idx_cache_type (cache_type),
    INDEX idx_cache_expires (expires_at),
    INDEX idx_cache_priority (priority DESC)
);
```

---

## 索引策略

### 4.1 索引设计原则

1. **查询优化**: 针对高频查询场景
2. **写入平衡**: 避免过度索引影响写入性能  
3. **存储优化**: 合理使用部分索引和表达式索引
4. **维护成本**: 定期评估索引使用率

### 4.2 核心索引

```sql
-- 主键索引 (自动创建)
-- 外键索引 (自动创建)

-- 查询优化索引
CREATE INDEX idx_drawings_lottery_date ON lottery_drawings(lottery_type_id, draw_date DESC);
CREATE INDEX idx_drawings_number_gin ON lottery_drawings USING GIN(winning_numbers);
CREATE INDEX idx_drawings_special_gin ON lottery_drawings USING GIN(special_numbers);

-- 预测查询索引
CREATE INDEX idx_predictions_strategy_target ON prediction_results(strategy_id, target_draw_date);
CREATE INDEX idx_predictions_lottery_date ON prediction_results(lottery_type_id, prediction_date DESC);
CREATE INDEX idx_predictions_winner ON prediction_results(is_winner, prize_amount DESC) WHERE is_winner = true;

-- 特征查询索引
CREATE INDEX idx_features_lottery_drawing ON analysis_features(lottery_type_id, drawing_id);
CREATE INDEX idx_features_type_name ON analysis_features(feature_type, feature_name);

-- 时间范围索引
CREATE INDEX idx_drawings_date_range ON lottery_drawings(draw_date) 
    WHERE draw_date >= CURRENT_DATE - INTERVAL '2 years';

-- 复合索引
CREATE INDEX idx_strategy_performance ON prediction_strategies(algorithm_type, accuracy_rate DESC);
```

### 4.3 全文搜索索引

```sql
-- 全文搜索支持
CREATE INDEX idx_drawings_search ON lottery_drawings 
    USING gin(to_tsvector('english', draw_number || ' ' || COALESCE(metadata->>'notes', '')));

-- 数字数组搜索优化
CREATE INDEX idx_numbers_similarity ON lottery_drawings 
    USING gin (winning_numbers gin__int_ops);
```

---

## 数据迁移方案

### 5.1 迁移策略

#### 5.1.1 分阶段迁移

```bash
# 阶段1: 结构迁移
psql -d lottery_db -f migrations/001_create_tables.sql

# 阶段2: 数据导入
python scripts/import_historical_data.py --source=csv --batch-size=1000

# 阶段3: 数据验证
python scripts/validate_data.py --check=consistency

# 阶段4: 索引创建
psql -d lottery_db -f migrations/002_create_indexes.sql

# 阶段5: 性能测试
python scripts/benchmark_queries.py
```

#### 5.1.2 回滚方案

```sql
-- 创建回滚点
CREATE SCHEMA backup_$(date +%Y%m%d_%H%M%S);

-- 备份数据
CREATE TABLE backup_$(date +%Y%m%d_%H%M%S).lottery_drawings_backup AS 
SELECT * FROM lottery_drawings;

-- 回滚操作
DROP TABLE IF EXISTS lottery_drawings;
ALTER TABLE backup_$(date +%Y%m%d_%H%M%S).lottery_drawings_backup 
RENAME TO lottery_drawings;
```

### 5.2 数据验证规则

```sql
-- 数据一致性检查
CREATE OR REPLACE FUNCTION validate_lottery_data()
RETURNS TABLE (
    issue_description TEXT,
    affected_records BIGINT
) AS $$
BEGIN
    -- 检查重复开奖记录
    RETURN QUERY
    SELECT 'Duplicate draw numbers'::TEXT, 
           COUNT(*)::BIGINT
    FROM lottery_drawings
    GROUP BY lottery_type_id, draw_number
    HAVING COUNT(*) > 1;
    
    -- 检查号码范围
    RETURN QUERY
    SELECT 'Numbers out of range'::TEXT,
           COUNT(*)::BIGINT
    FROM lottery_drawings ld
    JOIN lottery_types lt ON ld.lottery_type_id = lt.id
    WHERE EXISTS (
        SELECT 1 FROM unnest(ld.winning_numbers) num
        WHERE num < lt.main_range_start OR num > lt.main_range_end
    );
END;
$$ LANGUAGE plpgsql;
```

---

## 备份恢复策略

### 6.1 备份策略

#### 6.1.1 备份频率

| 备份类型 | 频率 | 保留时间 | 存储位置 |
|---|---|---|---|
| **完全备份** | 每日 2:00 | 30天 | 本地+云端 |
| **增量备份** | 每小时 | 7天 | 本地 |
| **实时备份** | 事务级 | 24小时 | 云端 |
| **归档备份** | 每月 | 永久 | 冷存储 |

#### 6.1.2 自动化备份脚本

```bash
#!/bin/bash
# backup.sh - 数据库备份脚本

BACKUP_DIR="/backup/lottery/$(date +%Y%m%d)"
PG_USER="lottery_user"
PG_DB="lottery_db"
S3_BUCKET="lottery-backups"

# 创建备份目录
mkdir -p $BACKUP_DIR

# PostgreSQL备份
pg_dump -U $PG_USER -h localhost -d $PG_DB \
    --format=custom \
    --file="$BACKUP_DIR/lottery_full_$(date +%H%M%S).dump" \
    --verbose \
    --lock-wait-timeout=60000

# DuckDB备份
cp /data/lottery.db "$BACKUP_DIR/lottery_duckdb_$(date +%H%M%S).db"

# 上传到云端
aws s3 sync $BACKUP_DIR s3://$S3_BUCKET/$(date +%Y%m%d)/

# 清理旧备份
find /backup/lottery -type d -mtime +30 -exec rm -rf {} \;
```

### 6.2 恢复策略

#### 6.2.1 恢复流程

```bash
#!/bin/bash
# restore.sh - 数据库恢复脚本

RESTORE_DATE=$1
BACKUP_TYPE=$2

if [ -z "$RESTORE_DATE" ]; then
    echo "Usage: $0 YYYYMMDD [full|incremental]"
    exit 1
fi

# 停止应用服务
systemctl stop lottery-app

# 恢复PostgreSQL
pg_restore -U lottery_user -d lottery_db \
    /backup/lottery/$RESTORE_DATE/lottery_full_*.dump

# 恢复DuckDB
cp /backup/lottery/$RESTORE_DATE/lottery_duckdb_*.db /data/lottery.db

# 验证数据完整性
psql -d lottery_db -c "SELECT validate_lottery_data();"

# 重启应用服务
systemctl start lottery-app
```

#### 6.2.2 灾难恢复

```sql
-- 创建灾难恢复脚本
CREATE OR REPLACE FUNCTION create_disaster_recovery_plan()
RETURNS JSON AS $$
DECLARE
    recovery_plan JSON;
BEGIN
    SELECT json_build_object(
        'timestamp', CURRENT_TIMESTAMP,
        'database_size', pg_database_size(current_database()),
        'table_counts', (
            SELECT json_object_agg(tablename, row_count)
            FROM (
                SELECT schemaname||'.'||tablename as tablename, 
                       n_live_tup as row_count
                FROM pg_stat_user_tables
            ) t
        ),
        'backup_locations', json_build_array(
            '/backup/lottery/latest',
            's3://lottery-backups/latest',
            '/cold-storage/lottery/monthly'
        )
    ) INTO recovery_plan;
    
    RETURN recovery_plan;
END;
$$ LANGUAGE plpgsql;
```

---

## 性能优化

### 7.1 查询优化

#### 7.1.1 慢查询监控

```sql
-- 创建慢查询日志表
CREATE TABLE slow_query_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    query_hash VARCHAR(64) NOT NULL,
    query_text TEXT NOT NULL,
    execution_time_ms INTEGER NOT NULL,
    execution_date TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    row_count BIGINT,
    user_agent VARCHAR(255)
);

-- 自动记录慢查询
CREATE OR REPLACE FUNCTION log_slow_query()
RETURNS event_trigger AS $$
DECLARE
    query_text TEXT;
    exec_time INTEGER;
BEGIN
    -- 获取查询信息和执行时间
    SELECT query, execution_time INTO query_text, exec_time
    FROM pg_stat_activity 
    WHERE pid = pg_backend_pid();
    
    -- 如果执行时间超过阈值，记录日志
    IF exec_time > 1000 THEN  -- 1秒
        INSERT INTO slow_query_log (query_hash, query_text, execution_time_ms)
        VALUES (
            md5(query_text),
            query_text,
            exec_time
        );
    END IF;
END;
$$ LANGUAGE plpgsql;
```

#### 7.1.2 查询优化示例

```sql
-- 优化前：慢查询
SELECT * FROM lottery_drawings 
WHERE lottery_type_id = '123e4567-e89b-12d3-a456-426614174000'
ORDER BY draw_date DESC
LIMIT 100;

-- 优化后：使用覆盖索引
SELECT id, draw_number, draw_date, winning_numbers, special_numbers
FROM lottery_drawings 
WHERE lottery_type_id = '123e4567-e89b-12d3-a456-426614174000'
ORDER BY draw_date DESC
LIMIT 100;

-- 优化前：复杂的聚合查询
SELECT 
    lottery_type_id,
    AVG(jackpot_amount) as avg_jackpot,
    COUNT(*) as draw_count
FROM lottery_drawings
WHERE draw_date >= CURRENT_DATE - INTERVAL '1 year'
GROUP BY lottery_type_id;

-- 优化后：使用物化视图
CREATE MATERIALIZED VIEW lottery_statistics_1y AS
SELECT 
    lottery_type_id,
    AVG(jackpot_amount) as avg_jackpot,
    COUNT(*) as draw_count,
    MAX(draw_date) as latest_draw
FROM lottery_drawings
WHERE draw_date >= CURRENT_DATE - INTERVAL '1 year'
GROUP BY lottery_type_id;

CREATE INDEX idx_stats_1y_lottery ON lottery_statistics_1y(lottery_type_id);
```

### 7.2 存储优化

#### 7.2.1 数据压缩

```sql
-- 启用列式存储压缩
ALTER TABLE lottery_drawings SET (toast_tuple_target = 128);

-- 压缩历史数据
CREATE TABLE lottery_drawings_compressed (LIKE lottery_drawings INCLUDING ALL);

-- 压缩策略：按年份压缩旧数据
INSERT INTO lottery_drawings_compressed
SELECT * FROM lottery_drawings
WHERE draw_date < CURRENT_DATE - INTERVAL '2 years';

-- 压缩率检查
SELECT 
    schemaname,
    tablename,
    pg_size_pretty(pg_total_relation_size(schemaname||'.'||tablename)) as size,
    pg_size_pretty(pg_total_relation_size(schemaname||'.'||tablename||'_compressed')) as compressed_size
FROM pg_tables 
WHERE tablename LIKE 'lottery_%';
```

#### 7.2.2 分区策略

```sql
-- 按时间分区
CREATE TABLE lottery_drawings_2024_08 PARTITION OF lottery_drawings
    FOR VALUES FROM ('2024-08-01') TO ('2024-09-01');

-- 自动创建未来分区
CREATE OR REPLACE FUNCTION create_monthly_partitions()
RETURNS void AS $$
DECLARE
    start_date date;
    end_date date;
    partition_name text;
BEGIN
    FOR i IN 0..11 LOOP
        start_date := date_trunc('month', CURRENT_DATE + (i || ' months')::interval);
        end_date := start_date + '1 month'::interval;
        partition_name := 'lottery_drawings_' || to_char(start_date, 'YYYY_MM');
        
        EXECUTE format('CREATE TABLE IF NOT EXISTS %I PARTITION OF lottery_drawings 
                       FOR VALUES FROM (%L) TO (%L)', 
                       partition_name, start_date, end_date);
    END LOOP;
END;
$$ LANGUAGE plpgsql;
```

---

## 监控运维

### 8.1 数据库监控

#### 8.1.1 性能指标监控

```sql
-- 创建监控视图
CREATE VIEW db_performance_metrics AS
SELECT 
    schemaname,
    tablename,
    n_tup_ins as inserts,
    n_tup_upd as updates,
    n_tup_del as deletes,
    n_live_tup as live_tuples,
    n_dead_tup as dead_tuples,
    pg_size_pretty(pg_total_relation_size(schemaname||'.'||tablename)) as size,
    last_vacuum,
    last_autovacuum,
    last_analyze
FROM pg_stat_user_tables
ORDER BY pg_total_relation_size(schemaname||'.'||tablename) DESC;

-- 查询性能监控
CREATE VIEW query_performance_stats AS
SELECT 
    query,
    calls,
    total_time,
    mean_time,
    rows,
    100.0 * shared_blks_hit / nullif(shared_blks_hit + shared_blks_read, 0) AS hit_percent
FROM pg_stat_statements
ORDER BY total_time DESC
LIMIT 20;
```

#### 8.1.2 自动告警配置

```sql
-- 创建告警函数
CREATE OR REPLACE FUNCTION check_database_health()
RETURNS TABLE (alert_type TEXT, alert_message TEXT, severity TEXT) AS $$
BEGIN
    -- 检查磁盘空间
    IF (SELECT pg_database_size(current_database())) > 10 * 1024 * 1024 * 1024 THEN
        RETURN QUERY
        SELECT 'disk_space'::TEXT, 
               'Database size exceeds 10GB'::TEXT, 
               'warning'::TEXT;
    END IF;
    
    -- 检查表膨胀
    IF EXISTS (
        SELECT 1 FROM pg_stat_user_tables 
        WHERE n_dead_tup > n_live_tup * 0.1
    ) THEN
        RETURN QUERY
        SELECT 'table_bloat'::TEXT,
               'High dead tuple ratio detected'::TEXT,
               'warning'::TEXT;
    END IF;
    
    -- 检查长时间运行查询
    IF EXISTS (
        SELECT 1 FROM pg_stat_activity 
        WHERE state = 'active' 
        AND query_start < now() - interval '5 minutes'
    ) THEN
        RETURN QUERY
        SELECT 'long_running_query'::TEXT,
               'Long running query detected'::TEXT,
               'critical'::TEXT;
    END IF;
END;
$$ LANGUAGE plpgsql;
```

### 8.2 自动化运维

#### 8.2.1 定期维护任务

```sql
-- 创建维护任务调度
CREATE OR REPLACE FUNCTION schedule_maintenance_tasks()
RETURNS void AS $$
BEGIN
    -- 每日数据清理
    PERFORM cron.schedule('daily_cleanup', '0 2 * * *', 
        'DELETE FROM cache_data WHERE expires_at < now()');
    
    -- 每周统计更新
    PERFORM cron.schedule('weekly_stats', '0 3 * * 0',
        'REFRESH MATERIALIZED VIEW lottery_statistics_1y');
    
    -- 每月分区创建
    PERFORM cron.schedule('monthly_partitions', '0 1 1 * *',
        'SELECT create_monthly_partitions()');
    
    -- 每季度索引维护
    PERFORM cron.schedule('quarterly_maintenance', '0 4 1 */3 *',
        'VACUUM ANALYZE lottery_drawings; VACUUM ANALYZE prediction_results;');
END;
$$ LANGUAGE plpgsql;
```

#### 8.2.2 健康检查API

```sql
-- 创建健康检查端点
CREATE OR REPLACE FUNCTION health_check()
RETURNS JSON AS $$
DECLARE
    health_status JSON;
BEGIN
    SELECT json_build_object(
        'status', 'healthy',
        'timestamp', CURRENT_TIMESTAMP,
        'database', json_build_object(
            'connected', true,
            'size', pg_database_size(current_database()),
            'tables', (
                SELECT COUNT(*) FROM pg_stat_user_tables
            ),
            'last_vacuum', (
                SELECT MAX(last_vacuum) FROM pg_stat_user_tables
            )
        ),
        'performance', json_build_object(
            'slow_queries', (
                SELECT COUNT(*) FROM slow_query_log 
                WHERE execution_date > now() - interval '1 hour'
            ),
            'cache_hit_ratio', (
                SELECT round(100.0 * blks_hit / (blks_hit + blks_read), 2)
                FROM pg_stat_database 
                WHERE datname = current_database()
            )
        )
    ) INTO health_status;
    
    RETURN health_status;
END;
$$ LANGUAGE plpgsql;
```

---

## 附录

### A. 数据库配置参数

#### A.1 PostgreSQL配置

```sql
-- 性能优化配置
shared_buffers = 256MB
effective_cache_size = 1GB
maintenance_work_mem = 64MB
checkpoint_completion_target = 0.9
wal_buffers = 16MB
default_statistics_target = 100
random_page_cost = 1.1
effective_io_concurrency = 200
work_mem = 4MB
min_wal_size = 1GB
max_wal_size = 4GB

-- 日志配置
log_destination = 'stderr'
logging_collector = on
log_directory = 'pg_log'
log_filename = 'postgresql-%Y-%m-%d_%H%M%S.log'
log_rotation_age = 1d
log_rotation_size = 100MB
log_min_duration_statement = 1000
```

#### A.2 DuckDB配置

```sql
-- DuckDB性能配置
SET memory_limit='1GB';
SET threads=4;
SET preserve_insertion_order=false;
SET checkpoint_threshold='1GB';
```

### B. 数据字典

#### B.1 表统计信息

```sql
-- 表数据量统计
SELECT 
    schemaname,
    tablename,
    n_tup_ins as inserts,
    n_tup_upd as updates,
    n_tup_del as deletes,
    n_live_tup as live_rows,
    pg_size_pretty(pg_total_relation_size(schemaname||'.'||tablename)) as size
FROM pg_stat_user_tables
ORDER BY pg_total_relation_size(schemaname||'.'||tablename) DESC;
```

#### B.2 索引统计信息

```sql
-- 索引使用情况
SELECT 
    schemaname,
    tablename,
    indexname,
    idx_tup_read,
    idx_tup_fetch,
    pg_size_pretty(pg_relation_size(indexrelname::regclass)) as size
FROM pg_stat_user_indexes
ORDER BY idx_tup_read DESC;
```

### C. 故障排除指南

#### C.1 常见问题

| 问题描述 | 可能原因 | 解决方案 |
|---|---|---|
| 查询缓慢 | 缺少索引 | 创建复合索引 |
| 写入阻塞 | 长事务 | 终止长事务 |
| 磁盘空间满 | 日志过多 | 清理日志文件 |
| 连接数超限 | 连接泄漏 | 调整连接池 |

#### C.2 监控查询

```sql
-- 当前活动查询
SELECT pid, now() - pg_stat_activity.query_start AS duration, query
FROM pg_stat_activity
WHERE (now() - pg_stat_activity.query_start) > interval '5 minutes';

-- 锁信息
SELECT blocked_locks.pid AS blocked_pid,
       blocked_activity.usename AS blocked_user,
       blocking_locks.pid AS blocking_pid,
       blocking_activity.usename AS blocking_user,
       blocked_activity.query AS blocked_statement,
       blocking_activity.query AS current_statement_in_blocking_process
FROM pg_catalog.pg_locks blocked_locks
JOIN pg_catalog.pg_activity blocked_activity ON blocked_activity.pid = blocked_locks.pid
JOIN pg_catalog.pg_locks blocking_locks ON blocking_locks.locktype = blocked_locks.locktype
AND blocking_locks.database IS NOT DISTINCT FROM blocked_locks.database
JOIN pg_catalog.pg_activity blocking_activity ON blocking_activity.pid = blocking_locks.pid
WHERE NOT blocked_locks.granted;
```

---

> **📌 重要提醒**  
> 1. 生产环境部署前请充分测试所有SQL脚本  
> 2. 定期审查和优化数据库性能  
> 3. 保持备份策略的定期验证  
> 4. 监控磁盘空间使用情况  

**文档状态**: ✅ 已批准  
**最后更新**: 2024-08-18  
**下次评审**: 2024-09-01