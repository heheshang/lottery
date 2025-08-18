-- 插入彩票类型种子数据
INSERT INTO lottery_types (
    name, display_name, category, total_numbers, special_numbers, 
    main_range_end, special_range_end, rules, is_system
) VALUES
('ssq', '双色球', 'welfare', 6, 1, 33, 16, 
 '{
    "selection_rules": {
        "main_numbers": {"min": 6, "max": 6},
        "special_numbers": {"min": 1, "max": 1}
    },
    "prize_rules": {
        "tiers": 6,
        "distribution": "pari-mutuel",
        "tier_requirements": [
            {"tier": 1, "main": 6, "special": 1},
            {"tier": 2, "main": 6, "special": 0},
            {"tier": 3, "main": 5, "special": 1},
            {"tier": 4, "main": 5, "special": 0},
            {"tier": 5, "main": 4, "special": 1},
            {"tier": 6, "main": 4, "special": 0}
        ]
    }
}', true),

('dlt', '大乐透', 'sports', 5, 2, 35, 12,
 '{
    "selection_rules": {
        "main_numbers": {"min": 5, "max": 5},
        "special_numbers": {"min": 2, "max": 2}
    },
    "prize_rules": {
        "tiers": 9,
        "distribution": "pari-mutuel",
        "tier_requirements": [
            {"tier": 1, "main": 5, "special": 2},
            {"tier": 2, "main": 5, "special": 1},
            {"tier": 3, "main": 5, "special": 0},
            {"tier": 4, "main": 4, "special": 2},
            {"tier": 5, "main": 4, "special": 1},
            {"tier": 6, "main": 3, "special": 2},
            {"tier": 7, "main": 4, "special": 0},
            {"tier": 8, "main": 3, "special": 1},
            {"tier": 9, "main": 2, "special": 2}
        ]
    }
}', true),

('fc3d', '福彩3D', 'welfare', 3, 0, 9, 9,
 '{
    "selection_rules": {
        "main_numbers": {"min": 3, "max": 3},
        "special_numbers": {"min": 0, "max": 0}
    },
    "prize_rules": {
        "tiers": 3,
        "distribution": "fixed",
        "tier_requirements": [
            {"tier": 1, "match": "exact", "order": "exact"},
            {"tier": 2, "match": "exact", "order": "any"},
            {"tier": 3, "match": "group", "order": "any"}
        ]
    }
}', true),

('pl3', '排列3', 'sports', 3, 0, 9, 9,
 '{
    "selection_rules": {
        "main_numbers": {"min": 3, "max": 3},
        "special_numbers": {"min": 0, "max": 0}
    },
    "prize_rules": {
        "tiers": 3,
        "distribution": "fixed",
        "tier_requirements": [
            {"tier": 1, "match": "exact", "order": "exact"},
            {"tier": 2, "match": "exact", "order": "any"},
            {"tier": 3, "match": "group", "order": "any"}
        ]
    }
}', true),

('pl5', '排列5', 'sports', 5, 0, 9, 9,
 '{
    "selection_rules": {
        "main_numbers": {"min": 5, "max": 5},
        "special_numbers": {"min": 0, "max": 0}
    },
    "prize_rules": {
        "tiers": 1,
        "distribution": "fixed",
        "tier_requirements": [
            {"tier": 1, "match": "exact", "order": "exact"}
        ]
    }
}', true);

-- 插入预测策略种子数据
INSERT INTO prediction_strategies (
    name, algorithm_type, description, parameters, hyperparameters, 
    feature_config, accuracy_rate, is_system, version
) VALUES
('随机森林-基础版', 'random_forest', '基于随机森林的基础预测策略', 
 '{
    "n_estimators": 100,
    "max_depth": 10,
    "min_samples_split": 2,
    "min_samples_leaf": 1,
    "random_state": 42
 }',
 '{
    "cv_folds": 5,
    "scoring": "accuracy",
    "n_jobs": -1
 }',
 '{
    "features": ["frequency", "trend", "statistical"],
    "window_size": 50,
    "include_special": true
 }',
 62.5, true, '1.0.0'),

('LSTM-时间序列', 'lstm', '基于LSTM神经网络的时间序列预测', 
 '{
    "hidden_size": 128,
    "num_layers": 2,
    "dropout": 0.2,
    "epochs": 100,
    "batch_size": 32,
    "learning_rate": 0.001
 }',
 '{
    "optimizer": "adam",
    "loss_function": "mse",
    "validation_split": 0.2,
    "early_stopping": true
 }',
 '{
    "features": ["temporal", "frequency", "pattern"],
    "sequence_length": 30,
    "include_time_features": true
 }',
 67.8, true, '1.0.0'),

('统计分析-频率法', 'statistical', '基于统计频率的简单预测方法', 
 '{
    "window_size": 50,
    "weight_function": "linear",
    "smoothing_factor": 0.1,
    "confidence_threshold": 0.6
 }',
 '{
    "min_samples": 100,
    "significance_level": 0.05
 }',
 '{
    "features": ["frequency", "hot_cold", "pattern"],
    "analysis_period": 100
 }',
 58.3, true, '1.0.0'),

('ARIMA-时间序列', 'arima', '基于ARIMA模型的时间序列分析', 
 '{
    "p": 2,
    "d": 1,
    "q": 2,
    "seasonal": false,
    "trend": "c"
 }',
 '{
    "method": "mle",
    "max_iter": 1000,
    "tolerance": 1e-06
 }',
 '{
    "features": ["temporal", "trend"],
    "differencing": true,
    "seasonal_analysis": false
 }',
 55.2, true, '1.0.0'),

('神经网络-深度学习', 'neural_network', '基于深度神经网络的预测模型', 
 '{
    "layers": [128, 64, 32],
    "activation": "relu",
    "dropout": 0.3,
    "epochs": 200,
    "batch_size": 64,
    "learning_rate": 0.001
 }',
 '{
    "optimizer": "adam",
    "loss_function": "categorical_crossentropy",
    "metrics": ["accuracy"],
    "validation_split": 0.2
 }',
 '{
    "features": ["frequency", "statistical", "pattern", "temporal"],
    "feature_scaling": true,
    "dimensionality_reduction": "pca"
 }',
 69.1, true, '1.0.0'),

('混合模型-集成', 'hybrid', '多种算法的集成预测模型', 
 '{
    "models": ["random_forest", "lstm", "neural_network"],
    "weights": [0.3, 0.4, 0.3],
    "ensemble_method": "weighted_average",
    "confidence_threshold": 0.65
 }',
 '{
    "cv_folds": 10,
    "scoring": "f1_weighted",
    "optimization": "bayesian"
 }',
 '{
    "features": ["all"],
    "feature_selection": true,
    "ensemble_voting": "soft"
 }',
 71.4, true, '1.0.0');

-- 插入示例历史开奖数据
INSERT INTO lottery_drawings (
    lottery_type_id, draw_number, draw_date, winning_numbers, special_numbers, 
    jackpot_amount, sales_amount, data_source, verification_status
) VALUES
-- 双色球示例数据
((SELECT id FROM lottery_types WHERE name = 'ssq'), '2024080', '2024-08-18', ARRAY[3,8,17,21,25,32], ARRAY[10], 5000000.00, 45000000.00, '500.com', 'verified'),
((SELECT id FROM lottery_types WHERE name = 'ssq'), '2024079', '2024-08-15', ARRAY[5,12,19,24,28,33], ARRAY[15], 4800000.00, 43000000.00, '500.com', 'verified'),
((SELECT id FROM lottery_types WHERE name = 'ssq'), '2024078', '2024-08-13', ARRAY[1,9,14,22,27,31], ARRAY[7], 4600000.00, 41000000.00, '500.com', 'verified'),
((SELECT id FROM lottery_types WHERE name = 'ssq'), '2024077', '2024-08-11', ARRAY[2,11,16,23,26,30], ARRAY[12], 4400000.00, 39000000.00, '500.com', 'verified'),
((SELECT id FROM lottery_types WHERE name = 'ssq'), '2024076', '2024-08-08', ARRAY[4,13,18,20,29,35], ARRAY[8], 4200000.00, 37000000.00, '500.com', 'verified'),

-- 大乐透示例数据
((SELECT id FROM lottery_types WHERE name = 'dlt'), '24080', '2024-08-17', ARRAY[5,12,20,26,33], ARRAY[3,9], 8000000.00, 32000000.00, '500.com', 'verified'),
((SELECT id FROM lottery_types WHERE name = 'dlt'), '24079', '2024-08-14', ARRAY[3,15,21,28,34], ARRAY[2,11], 7800000.00, 30000000.00, '500.com', 'verified'),
((SELECT id FROM lottery_types WHERE name = 'dlt'), '24078', '2024-08-12', ARRAY[7,14,22,29,35], ARRAY[4,8], 7600000.00, 28000000.00, '500.com', 'verified'),

-- 福彩3D示例数据
((SELECT id FROM lottery_types WHERE name = 'fc3d'), '2024218', '2024-08-18', ARRAY[3,5,8], NULL, 1040.00, 25000000.00, '500.com', 'verified'),
((SELECT id FROM lottery_types WHERE name = 'fc3d'), '2024217', '2024-08-17', ARRAY[1,4,9], NULL, 1040.00, 24000000.00, '500.com', 'verified'),
((SELECT id FROM lottery_types WHERE name = 'fc3d'), '2024216', '2024-08-16', ARRAY[2,6,7], NULL, 1040.00, 23000000.00, '500.com', 'verified');

-- 插入分析特征示例数据
INSERT INTO analysis_features (
    lottery_type_id, drawing_id, feature_type, feature_name, 
    feature_data, feature_vector, data_points, calculation_time_ms
) SELECT 
    lt.id, 
    ld.id,
    'frequency',
    'number_frequency_30d',
    jsonb_build_object(
        'period', 30,
        'frequencies', jsonb_object_agg(n.num, freq.freq)
    ),
    array_agg(freq.freq ORDER BY n.num),
    30,
    150
FROM lottery_types lt
JOIN lottery_drawings ld ON ld.lottery_type_id = lt.id
CROSS JOIN generate_series(1, 33) AS n(num)
LEFT JOIN LATERAL (
    SELECT num, COUNT(*)::float as freq
    FROM lottery_drawings ld2
    WHERE ld2.lottery_type_id = lt.id 
    AND ld2.draw_date >= ld.draw_date - INTERVAL '30 days'
    AND ld2.draw_date <= ld.draw_date
    AND n.num = ANY(ld2.winning_numbers)
) freq ON true
WHERE lt.name = 'ssq'
AND ld.draw_date >= '2024-08-01'
GROUP BY lt.id, ld.id
LIMIT 10;

-- 插入预测结果示例数据
INSERT INTO prediction_results (
    strategy_id, lottery_type_id, predicted_numbers, predicted_special_numbers, 
    confidence_scores, target_draw_date, accuracy_score, match_count, 
    special_match_count, prediction_type, computation_time_ms
) SELECT 
    s.id,
    lt.id,
    ARRAY[3,8,15,22,28,31],
    ARRAY[10],
    ARRAY[0.85,0.78,0.82,0.75,0.80,0.77,0.83],
    '2024-08-20',
    0.67,
    3,
    1,
    'standard',
    1200
FROM prediction_strategies s
JOIN lottery_types lt ON lt.name = 'ssq'
WHERE s.algorithm_type = 'random_forest'
LIMIT 1;