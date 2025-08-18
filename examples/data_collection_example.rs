use lottery_prediction::lottery::services::DataCollector;
use lottery_prediction::lottery::models::LotteryType;
use sqlx::postgres::PgPoolOptions;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 设置日志
    env_logger::init();
    
    // 连接数据库
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://localhost/lottery".to_string());
    
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;
    
    // 创建数据采集器
    let collector = DataCollector::new(pool.clone(), None);
    
    println!("Starting lottery data collection...");
    
    // 收集所有数据源的数据
    let all_data = collector.collect_all_data().await?;
    println!("Collected {} records from all sources", all_data.len());
    
    // 验证数据
    let validated_data = collector.validate_data(&all_data).await?;
    println!("Validated {} records", validated_data.len());
    
    // 保存数据
    let saved_count = collector.save_data(validated_data).await?;
    println!("Successfully saved {} records", saved_count);
    
    // 检查缺失数据
    let start_date = chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
    let end_date = chrono::NaiveDate::from_ymd_opt(2024, 12, 31).unwrap();
    
    for lottery_type in [LotteryType::Ssq, LotteryType::Dlt, LotteryType::Fc3d] {
        let missing_dates = collector.get_missing_dates(
            &lottery_type,
            start_date,
            end_date,
        ).await?;
        
        println!("Missing {} dates for {}", missing_dates.len(), lottery_type);
    }
    
    Ok(())
}