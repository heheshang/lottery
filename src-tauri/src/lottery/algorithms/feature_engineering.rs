use crate::lottery::algorithms::traits::{FeatureConfig, FeatureExtractor, TrainingData};
use crate::lottery::errors::LotteryResult as Result;
use crate::lottery::models::LotteryDrawing;
use crate::lottery::models::LotteryType;
use chrono::Datelike;

pub struct LotteryFeatureExtractor;

impl FeatureExtractor for LotteryFeatureExtractor {
    fn extract_features(
        &self,
        drawings: &[LotteryDrawing],
        config: &FeatureConfig,
    ) -> Result<TrainingData> {
        if drawings.is_empty() {
            return Err(crate::lottery::errors::LotteryError::InvalidParameter(
                "No drawings provided for feature extraction".to_string(),
            ));
        }

        let mut features = Vec::new();
        let mut targets = Vec::new();
        let mut special_targets = Vec::new();
        let mut weights = Vec::new();

        let lottery_type = &drawings[0].lottery_type;
        let max_number = self.get_max_number(lottery_type);
        let special_max = self.get_special_max(lottery_type);

        for (i, drawing) in drawings.iter().enumerate() {
            if i < config.window_size {
                continue; // Skip first few for windowing
            }

            let historical_window = &drawings[i - config.window_size..i];
            let feature_vector = self.extract_single_features(drawing, historical_window)?;

            features.push(feature_vector);
            targets.push(drawing.winning_numbers.clone());
            
            if config.include_special_numbers {
                special_targets.push(drawing.special_numbers.clone().unwrap_or_default());
            }

            // Weight recent data more heavily
            let weight = (i as f64 + 1.0) / drawings.len() as f64;
            weights.push(weight);
        }

        Ok(TrainingData {
            features,
            targets,
            special_targets: if config.include_special_numbers {
                Some(special_targets)
            } else {
                None
            },
            weights: Some(weights),
        })
    }

    fn extract_single_features(
        &self,
        drawing: &LotteryDrawing,
        historical_data: &[LotteryDrawing],
    ) -> Result<Vec<f64>> {
        let mut features = Vec::new();
        let lottery_type = &drawing.lottery_type;
        let max_number = self.get_max_number(lottery_type);
        let special_max = self.get_special_max(lottery_type);

        // 1. Frequency analysis
        features.extend(self.calculate_frequency_features(historical_data, max_number)?);

        // 2. Trend analysis
        features.extend(self.calculate_trend_features(historical_data, max_number)?);

        // 3. Statistical analysis
        features.extend(self.calculate_statistical_features(historical_data)?);

        // 4. Pattern analysis
        features.extend(self.calculate_pattern_features(historical_data)?);

        // 5. Temporal analysis
        features.extend(self.calculate_temporal_features(drawing)?);

        // 6. Hot and cold numbers
        features.extend(self.calculate_hot_cold_features(historical_data, max_number)?);

        // 7. Gap analysis
        features.extend(self.calculate_gap_features(historical_data, max_number)?);

        // 8. Sum analysis
        features.extend(self.calculate_sum_features(historical_data)?);

        // 9. Parity analysis
        features.extend(self.calculate_parity_features(historical_data)?);

        // 10. Special numbers features
        features.extend(self.calculate_special_features(historical_data, special_max)?);

        Ok(features)
    }

    fn get_feature_names(&self) -> Vec<String> {
        let mut names = Vec::new();
        
        // Frequency features
        for i in 1..=50 {
            names.push(format!("freq_{}", i));
        }
        
        // Trend features
        names.extend(vec![
            "trend_up".to_string(),
            "trend_down".to_string(),
            "trend_stable".to_string(),
        ]);
        
        // Statistical features
        names.extend(vec![
            "mean".to_string(),
            "std".to_string(),
            "min".to_string(),
            "max".to_string(),
            "median".to_string(),
        ]);
        
        // Pattern features
        names.extend(vec![
            "consecutive_count".to_string(),
            "odd_count".to_string(),
            "even_count".to_string(),
            "prime_count".to_string(),
        ]);
        
        // Temporal features
        names.extend(vec![
            "day_of_week".to_string(),
            "day_of_month".to_string(),
            "month".to_string(),
            "is_weekend".to_string(),
        ]);
        
        names
    }

    fn validate_features(&self, features: &[f64]) -> Result<bool> {
        if features.is_empty() {
            return Ok(false);
        }

        // Check for NaN or infinite values
        for &value in features {
            if value.is_nan() || value.is_infinite() {
                return Ok(false);
            }
        }

        Ok(true)
    }
}

impl LotteryFeatureExtractor {
    fn get_max_number(&self, lottery_type: &LotteryType) -> usize {
        match lottery_type {
            LotteryType::Ssq => 33,
            LotteryType::Dlt => 35,
            LotteryType::Fc3d => 9,
            LotteryType::Pl3 => 9,
            LotteryType::Pl5 => 9,
            LotteryType::Custom => 49,
        }
    }

    fn get_special_max(&self, lottery_type: &LotteryType) -> usize {
        match lottery_type {
            LotteryType::Ssq => 16,
            LotteryType::Dlt => 12,
            LotteryType::Fc3d => 0,
            LotteryType::Pl3 => 0,
            LotteryType::Pl5 => 0,
            LotteryType::Custom => 16,
        }
    }

    fn calculate_frequency_features(
        &self,
        historical_data: &[LotteryDrawing],
        max_number: usize,
    ) -> Result<Vec<f64>> {
        let mut frequencies = vec![0.0; max_number + 1];
        
        for drawing in historical_data {
            for &number in &drawing.winning_numbers {
                if number <= max_number as u32 {
                    frequencies[number as usize] += 1.0;
                }
            }
        }

        // Normalize frequencies
        let total = historical_data.len() as f64;
        for freq in &mut frequencies[1..] {
            *freq /= total;
        }

        Ok(frequencies[1..].to_vec())
    }

    fn calculate_trend_features(
        &self,
        historical_data: &[LotteryDrawing],
        max_number: usize,
    ) -> Result<Vec<f64>> {
        if historical_data.len() < 2 {
            return Ok(vec![0.0, 0.0, 1.0]);
        }

        let mut up_trend = 0.0;
        let mut down_trend = 0.0;
        let mut stable = 0.0;

        for i in 1..historical_data.len() {
            let prev_avg = self.calculate_average(&historical_data[i - 1].winning_numbers);
            let curr_avg = self.calculate_average(&historical_data[i].winning_numbers);

            if curr_avg > prev_avg {
                up_trend += 1.0;
            } else if curr_avg < prev_avg {
                down_trend += 1.0;
            } else {
                stable += 1.0;
            }
        }

        let total = historical_data.len() as f64 - 1.0;
        Ok(vec![up_trend / total, down_trend / total, stable / total])
    }

    fn calculate_statistical_features(&self, historical_data: &[LotteryDrawing]) -> Result<Vec<f64>> {
        let all_numbers: Vec<u32> = historical_data
            .iter()
            .flat_map(|d| d.winning_numbers.clone())
            .collect();

        if all_numbers.is_empty() {
            return Ok(vec![0.0, 0.0, 0.0, 0.0, 0.0]);
        }

        let numbers_f64: Vec<f64> = all_numbers.iter().map(|&n| n as f64).collect();
        
        let mean = numbers_f64.iter().sum::<f64>() / numbers_f64.len() as f64;
        let variance = numbers_f64.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / numbers_f64.len() as f64;
        let std = variance.sqrt();
        let min = *numbers_f64.iter().fold(None, |min, x| match min {
            None => Some(x),
            Some(y) => Some(if x < y { x } else { y }),
        }).unwrap_or(&0.0);
        let max = *numbers_f64.iter().fold(None, |max, x| match max {
            None => Some(x),
            Some(y) => Some(if x > y { x } else { y }),
        }).unwrap_or(&0.0);
        
        let sorted = {
            let mut sorted = numbers_f64.clone();
            sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
            sorted
        };
        
        let median = if sorted.len() % 2 == 0 {
            (sorted[sorted.len() / 2 - 1] + sorted[sorted.len() / 2]) / 2.0
        } else {
            sorted[sorted.len() / 2]
        };

        Ok(vec![mean, std, min, max, median])
    }

    fn calculate_pattern_features(&self, historical_data: &[LotteryDrawing]) -> Result<Vec<f64>> {
        let mut consecutive_count = 0.0;
        let mut odd_count = 0.0;
        let mut even_count = 0.0;
        let mut prime_count = 0.0;

        for drawing in historical_data {
            let numbers = &drawing.winning_numbers;
            
            // Check for consecutive numbers
            for window in numbers.windows(2) {
                if window[1] == window[0] + 1 {
                    consecutive_count += 1.0;
                }
            }
            
            // Count odd, even, and prime numbers
            for &number in numbers {
                if number % 2 == 1 {
                    odd_count += 1.0;
                } else {
                    even_count += 1.0;
                }
                
                if self.is_prime(number) {
                    prime_count += 1.0;
                }
            }
        }

        let total = historical_data.len() as f64;
        Ok(vec![consecutive_count / total, odd_count / total, even_count / total, prime_count / total])
    }

    fn calculate_temporal_features(&self, drawing: &LotteryDrawing) -> Result<Vec<f64>> {
        let date = drawing.draw_date;
        let weekday = date.weekday().num_days_from_monday() as f64 / 7.0;
        let day_of_month = date.day() as f64 / 31.0;
        let month = date.month() as f64 / 12.0;
        let is_weekend = if date.weekday().number_from_monday() >= 6 {
            1.0
        } else {
            0.0
        };

        Ok(vec![weekday, day_of_month, month, is_weekend])
    }

    fn calculate_hot_cold_features(
        &self,
        historical_data: &[LotteryDrawing],
        max_number: usize,
    ) -> Result<Vec<f64>> {
        let mut hot_numbers = vec![0.0; max_number + 1];
        let mut cold_numbers = vec![0.0; max_number + 1];

        let recent_data = if historical_data.len() > 10 {
            &historical_data[historical_data.len() - 10..]
        } else {
            historical_data
        };

        let older_data = if historical_data.len() > 20 {
            &historical_data[..historical_data.len() - 10]
        } else {
            &historical_data[..]
        };

        // Calculate frequencies for both periods
        for drawing in recent_data {
            for &number in &drawing.winning_numbers {
                if number <= max_number as u32 {
                    hot_numbers[number as usize] += 1.0;
                }
            }
        }

        for drawing in older_data {
            for &number in &drawing.winning_numbers {
                if number <= max_number as u32 {
                    cold_numbers[number as usize] += 1.0;
                }
            }
        }

        // Normalize
        let hot_total = hot_numbers.iter().sum::<f64>();
        let cold_total = cold_numbers.iter().sum::<f64>();

        if hot_total > 0.0 {
            for val in &mut hot_numbers {
                *val /= hot_total;
            }
        }

        if cold_total > 0.0 {
            for val in &mut cold_numbers {
                *val /= cold_total;
            }
        }

        Ok(hot_numbers[1..]
            .iter()
            .zip(cold_numbers[1..].iter())
            .map(|(h, c)| h - c)
            .collect())
    }

    fn calculate_gap_features(
        &self,
        historical_data: &[LotteryDrawing],
        max_number: usize,
    ) -> Result<Vec<f64>> {
        let mut last_seen = vec![historical_data.len() as f64; max_number + 1];
        let mut gap_features = vec![0.0; max_number + 1];

        for (i, drawing) in historical_data.iter().enumerate() {
            for &number in &drawing.winning_numbers {
                if number <= max_number as u32 {
                    let idx = number as usize;
                    gap_features[idx] = (i as f64 - last_seen[idx]).abs();
                    last_seen[idx] = i as f64;
                }
            }
        }

        // Normalize gaps
        let max_gap = gap_features.iter().cloned().fold(0.0, f64::max);
        if max_gap > 0.0 {
            for gap in &mut gap_features[1..] {
                *gap /= max_gap;
            }
        }

        Ok(gap_features[1..].to_vec())
    }

    fn calculate_sum_features(&self, historical_data: &[LotteryDrawing]) -> Result<Vec<f64>> {
        let sums: Vec<f64> = historical_data
            .iter()
            .map(|d| d.winning_numbers.iter().sum::<u32>() as f64)
            .collect();

        if sums.is_empty() {
            return Ok(vec![0.0, 0.0, 0.0]);
        }

        let mean = sums.iter().sum::<f64>() / sums.len() as f64;
        let variance = sums.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / sums.len() as f64;
        let std = variance.sqrt();

        Ok(vec![mean, std, *sums.last().unwrap_or(&0.0)])
    }

    fn calculate_parity_features(&self, historical_data: &[LotteryDrawing]) -> Result<Vec<f64>> {
        let mut odd_ratios = Vec::new();
        let mut even_ratios = Vec::new();

        for drawing in historical_data {
            let numbers = &drawing.winning_numbers;
            let odd_count = numbers.iter().filter(|&&n| n % 2 == 1).count() as f64;
            let even_count = numbers.iter().filter(|&&n| n % 2 == 0).count() as f64;
            let total = numbers.len() as f64;

            odd_ratios.push(odd_count / total);
            even_ratios.push(even_count / total);
        }

        let odd_mean = odd_ratios.iter().sum::<f64>() / odd_ratios.len() as f64;
        let even_mean = even_ratios.iter().sum::<f64>() / even_ratios.len() as f64;

        Ok(vec![odd_mean, even_mean])
    }

    fn calculate_special_features(
        &self,
        historical_data: &[LotteryDrawing],
        special_max: usize,
    ) -> Result<Vec<f64>> {
        if special_max == 0 {
            return Ok(Vec::new());
        }

        let mut special_frequencies = vec![0.0; special_max + 1];

        for drawing in historical_data {
            if let Some(ref specials) = drawing.special_numbers {
                for &special in specials {
                    if special <= special_max as u32 {
                        special_frequencies[special as usize] += 1.0;
                    }
                }
            }
        }

        // Normalize
        let total = historical_data.len() as f64;
        if total > 0.0 {
            for freq in &mut special_frequencies[1..] {
                *freq /= total;
            }
        }

        Ok(special_frequencies[1..].to_vec())
    }

    fn calculate_average(&self, numbers: &[u32]) -> f64 {
        if numbers.is_empty() {
            0.0
        } else {
            numbers.iter().sum::<u32>() as f64 / numbers.len() as f64
        }
    }

    fn is_prime(&self, n: u32) -> bool {
        if n < 2 {
            return false;
        }
        for i in 2..=((n as f64).sqrt() as u32) {
            if n % i == 0 {
                return false;
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lottery::models::{LotteryDrawing, LotteryType};
    use chrono::NaiveDate;
    use uuid::Uuid;

    #[test]
    fn test_extract_features() {
        let extractor = LotteryFeatureExtractor;
        let config = FeatureConfig::default();

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

        let result = extractor.extract_features(&drawings, &config);
        assert!(result.is_ok());
        
        let training_data = result.unwrap();
        assert!(!training_data.features.is_empty());
    }

    #[test]
    fn test_extract_single_features() {
        let extractor = LotteryFeatureExtractor;
        let drawing = LotteryDrawing {
            id: Uuid::new_v4(),
            lottery_type: LotteryType::Ssq,
            draw_number: "2024001".to_string(),
            draw_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
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
        };

        let historical_data = vec![drawing.clone(); 5];
        let features = extractor.extract_single_features(&drawing, &historical_data).unwrap();
        assert!(!features.is_empty());
    }
}