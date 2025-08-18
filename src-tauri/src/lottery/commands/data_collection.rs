use crate::lottery::services::DataCollector;
use crate::lottery::models::LotteryType;
use crate::lottery::LotteryDrawing;
use sqlx::PgPool;
use std::collections::HashMap;

pub async fn collect_lottery_data(
    pool: PgPool,
    lottery_type: LotteryType,
    start_date: chrono::NaiveDate,
    end_date: chrono::NaiveDate,
) -> Result<Vec<LotteryDrawing>, crate::lottery::errors::LotteryError> {
    let collector = DataCollector::new(pool, None);
    
    // 获取缺失的日期
    let missing_dates = collector.get_missing_dates(
        &lottery_type,
        start_date,
        end_date,
    ).await?;
    
    if missing_dates.is_empty() {
        println!("No missing data for {} between {} and {}", 
            lottery_type, start_date, end_date);
        return Ok(Vec::new());
    }
    
    println!("Collecting data for {} missing dates", missing_dates.len());
    
    // 收集所有数据
    let all_data = collector.collect_all_data().await?;
    
    // 验证数据
    let validated_data = collector.validate_data(&all_data).await?;
    
    // 保存数据
    let saved_count = collector.save_data(validated_data.clone()).await?;
    
    println!("Successfully collected and saved {} lottery records", saved_count);
    
    Ok(validated_data)
}

pub async fn collect_all_lottery_data(
    pool: PgPool,
) -> Result<HashMap<LotteryType, usize>, crate::lottery::errors::LotteryError> {
    let collector = DataCollector::new(pool, None);
    
    let lottery_types = vec![
        LotteryType::Ssq,
        LotteryType::Dlt,
        LotteryType::Fc3d,
        LotteryType::Pl3,
        LotteryType::Pl5,
    ];
    
    let mut results = HashMap::new();
    
    for lottery_type in lottery_types {
        let data = collector.collect_all_data().await?;
        let validated_data = collector.validate_data(&data).await?;
        let saved_count = collector.save_data(validated_data).await?;
        
        results.insert(lottery_type, saved_count);
        
        println!("Collected {} records for {}", saved_count, lottery_type);
    }
    
    Ok(results)
}

pub async fn validate_existing_data(
    pool: PgPool,
    lottery_type: Option<LotteryType>,
) -> Result<( usize, usize), crate::lottery::errors::LotteryError> {
    let collector = DataCollector::new(pool, None);
    
    // 这里需要实现从数据库查询现有数据
    // 然后验证并更新状态
    
    Ok((0, 0)) // (total_records, invalid_records)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lottery::models::LotteryType;

    #[tokio::test]
    async fn test_collect_lottery_data() {
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(5)
            .connect("postgres://localhost/lottery_test")
            .await
            .unwrap();
        
        let result = collect_lottery_data(
            pool,
            LotteryType::Ssq,
            chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            chrono::NaiveDate::from_ymd_opt(2024, 1, 31).unwrap(),
        ).await;
        
        assert!(result.is_ok());
    }
}