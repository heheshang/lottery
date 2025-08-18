#[cfg(test)]
mod data_collection_tests {
    use super::*;
    use crate::lottery::models::LotteryType;
    use crate::lottery::services::DataCollector;
    use chrono::NaiveDate;
    use sqlx::postgres::PgPoolOptions;
    use tokio::time::timeout;

    async fn setup_test_db() -> sqlx::PgPool {
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://localhost/lottery_test".to_string());
        
        PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
            .expect("Failed to connect to test database")
    }

    #[tokio::test]
    async fn test_data_collector_creation() {
        let pool = setup_test_db().await;
        let collector = DataCollector::new(pool, None);
        
        assert!(!collector.config.sources.is_empty());
        assert_eq!(collector.config.retry_attempts, 3);
        assert_eq!(collector.config.batch_size, 100);
    }

    #[tokio::test]
    async fn test_html_parser_ssq() {
        let html = r#"
            <table class="tbl1">
                <tr>
                    <th>期号</th>
                    <th>开奖日期</th>
                    <th>红球</th>
                    <th>蓝球</th>
                    <th>奖金池</th>
                </tr>
                <tr>
                    <td>2024080</td>
                    <td>2024-08-18</td>
                    <td>03 08 17 21 25 32</td>
                    <td>10</td>
                    <td>5000000</td>
                </tr>
                <tr>
                    <td>2024079</td>
                    <td>2024-08-15</td>
                    <td>05 12 19 24 28 33</td>
                    <td>15</td>
                    <td>4800000</td>
                </tr>
            </table>
        "#;
        
        let collector = DataCollector::new(setup_test_db().await, None);
        let source = DataSource {
            name: "test".to_string(),
            url: "http://test.com".to_string(),
            lottery_type: LotteryType::Ssq,
            parser: ParserType::Html,
        };
        
        let result = collector.parse_html(html, &source);
        assert!(result.is_ok());
        
        let drawings = result.unwrap();
        assert_eq!(drawings.len(), 2);
        assert_eq!(drawings[0].draw_number, "2024080");
        assert_eq!(drawings[0].winning_numbers, vec![3, 8, 17, 21, 25, 32]);
        assert_eq!(drawings[0].special_numbers, Some(vec![10]));
    }

    #[tokio::test]
    async fn test_json_parser_dlt() {
        let json = r#"
            [
                {
                    "issue": "24080",
                    "date": "2024-08-17",
                    "front": "05,12,20,26,33",
                    "back": "03,09",
                    "money": 8000000.0
                },
                {
                    "issue": "24079",
                    "date": "2024-08-14",
                    "front": "03,15,21,28,34",
                    "back": "02,11",
                    "money": 7800000.0
                }
            ]
        "#;
        
        let collector = DataCollector::new(setup_test_db().await, None);
        let source = DataSource {
            name: "test".to_string(),
            url: "http://test.com".to_string(),
            lottery_type: LotteryType::Dlt,
            parser: ParserType::Json,
        };
        
        let result = collector.parse_json(json, &source);
        assert!(result.is_ok());
        
        let drawings = result.unwrap();
        assert_eq!(drawings.len(), 2);
        assert_eq!(drawings[0].draw_number, "24080");
        assert_eq!(drawings[0].winning_numbers, vec![5, 12, 20, 26, 33]);
        assert_eq!(drawings[0].special_numbers, Some(vec![3, 9]));
    }

    #[tokio::test]
    async fn test_xml_parser_ssq() {
        let xml = r#"
            <?xml version="1.0" encoding="UTF-8"?>
            <data>
                <record>
                    <issue>2024080</issue>
                    <date>2024-08-18</date>
                    <red>03,08,17,21,25,32</red>
                    <blue>10</blue>
                    <money>5000000</money>
                </record>
                <record>
                    <issue>2024079</issue>
                    <date>2024-08-15</date>
                    <red>05,12,19,24,28,33</red>
                    <blue>15</blue>
                    <money>4800000</money>
                </record>
            </data>
        "#;
        
        let collector = DataCollector::new(setup_test_db().await, None);
        let source = DataSource {
            name: "test".to_string(),
            url: "http://test.com".to_string(),
            lottery_type: LotteryType::Ssq,
            parser: ParserType::Xml,
        };
        
        let result = collector.parse_xml(xml, &source);
        assert!(result.is_ok());
        
        let drawings = result.unwrap();
        assert_eq!(drawings.len(), 2);
        assert_eq!(drawings[0].draw_number, "2024080");
        assert_eq!(drawings[0].winning_numbers, vec![3, 8, 17, 21, 25, 32]);
        assert_eq!(drawings[0].special_numbers, Some(vec![10]));
    }

    #[tokio::test]
    async fn test_regex_parser_fc3d() {
        let text = r#"
            2024218 2024-08-18 358
            2024217 2024-08-17 149
            2024216 2024-08-16 267
        "#;
        
        let collector = DataCollector::new(setup_test_db().await, None);
        let source = DataSource {
            name: "test".to_string(),
            url: "http://test.com".to_string(),
            lottery_type: LotteryType::Fc3d,
            parser: ParserType::Regex,
        };
        
        let result = collector.parse_regex(text, &source);
        assert!(result.is_ok());
        
        let drawings = result.unwrap();
        assert_eq!(drawings.len(), 3);
        assert_eq!(drawings[0].draw_number, "2024218");
        assert_eq!(drawings[0].winning_numbers, vec![3, 5, 8]);
    }

    #[tokio::test]
    async fn test_validate_single_drawing() {
        let pool = setup_test_db().await;
        let collector = DataCollector::new(pool, None);
        
        let valid_drawing = LotteryDrawing {
            id: uuid::Uuid::new_v4(),
            lottery_type: LotteryType::Ssq,
            draw_number: "2024001".to_string(),
            draw_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            draw_time: None,
            winning_numbers: vec![1, 2, 3, 4, 5, 6],
            special_numbers: Some(vec![7]),
            jackpot_amount: Some(1000000.0),
            sales_amount: None,
            prize_distribution: None,
            data_source: "test".to_string(),
            verification_status: "pending".to_string(),
            metadata: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            crawled_at: None,
        };
        
        let result = collector.validate_single_drawing(&valid_drawing).await;
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[tokio::test]
    async fn test_validate_invalid_drawing() {
        let pool = setup_test_db().await;
        let collector = DataCollector::new(pool, None);
        
        let invalid_drawing = LotteryDrawing {
            id: uuid::Uuid::new_v4(),
            lottery_type: LotteryType::Ssq,
            draw_number: "2024001".to_string(),
            draw_date: NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(), // 未来日期
            draw_time: None,
            winning_numbers: vec![1, 2, 3, 4, 5, 100], // 超出范围
            special_numbers: Some(vec![7]),
            jackpot_amount: Some(1000000.0),
            sales_amount: None,
            prize_distribution: None,
            data_source: "test".to_string(),
            verification_status: "pending".to_string(),
            metadata: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            crawled_at: None,
        };
        
        let result = collector.validate_single_drawing(&invalid_drawing).await;
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[tokio::test]
    async fn test_validate_data_batch() {
        let pool = setup_test_db().await;
        let collector = DataCollector::new(pool, None);
        
        let drawings = vec![
            LotteryDrawing {
                id: uuid::Uuid::new_v4(),
                lottery_type: LotteryType::Ssq,
                draw_number: "2024001".to_string(),
                draw_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                draw_time: None,
                winning_numbers: vec![1, 2, 3, 4, 5, 6],
                special_numbers: Some(vec![7]),
                jackpot_amount: Some(1000000.0),
                sales_amount: None,
                prize_distribution: None,
                data_source: "test".to_string(),
                verification_status: "pending".to_string(),
                metadata: None,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
                crawled_at: None,
            },
            LotteryDrawing {
                id: uuid::Uuid::new_v4(),
                lottery_type: LotteryType::Ssq,
                draw_number: "2024002".to_string(),
                draw_date: NaiveDate::from_ymd_opt(2024, 1, 3).unwrap(),
                draw_time: None,
                winning_numbers: vec![10, 20, 30, 40, 50, 60], // 超出范围
                special_numbers: Some(vec![70]),
                jackpot_amount: Some(1000000.0),
                sales_amount: None,
                prize_distribution: None,
                data_source: "test".to_string(),
                verification_status: "pending".to_string(),
                metadata: None,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
                crawled_at: None,
            },
        ];
        
        let result = collector.validate_data(&drawings).await;
        assert!(result.is_ok());
        
        let validated = result.unwrap();
        assert_eq!(validated.len(), 1); // 只有第一个有效
        assert_eq!(validated[0].draw_number, "2024001");
    }
}