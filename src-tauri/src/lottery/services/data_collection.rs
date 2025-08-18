use crate::lottery::errors::{LotteryError, Result};
use crate::lottery::models::{LotteryDrawing, LotteryType};
use chrono::{Duration, NaiveDate, Utc};
use regex::Regex;
use reqwest::Client;
use roxmltree::Document;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::collections::HashMap;
use tokio::time::{sleep, timeout};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataSource {
    pub name: String,
    pub url: String,
    pub lottery_type: LotteryType,
    pub parser: ParserType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParserType {
    Json,
    Html,
    Xml,
    Regex,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrawlConfig {
    pub sources: Vec<DataSource>,
    pub retry_attempts: u32,
    pub retry_delay: u64,
    pub timeout_seconds: u64,
    pub batch_size: usize,
}

impl Default for CrawlConfig {
    fn default() -> Self {
        Self {
            sources: vec![
                DataSource {
                    name: "500.com".to_string(),
                    url: "https://datachart.500.com/ssq/history/newinc/history.php".to_string(),
                    lottery_type: LotteryType::Ssq,
                    parser: ParserType::Html,
                },
                DataSource {
                    name: "500.com".to_string(),
                    url: "https://datachart.500.com/dlt/history/newinc/history.php".to_string(),
                    lottery_type: LotteryType::Dlt,
                    parser: ParserType::Html,
                },
                DataSource {
                    name: "lottery.gov.cn".to_string(),
                    url: "https://www.lottery.gov.cn/historykj/history.jspx".to_string(),
                    lottery_type: LotteryType::Ssq,
                    parser: ParserType::Json,
                },
            ],
            retry_attempts: 3,
            retry_delay: 1000,
            timeout_seconds: 30,
            batch_size: 100,
        }
    }
}

pub struct DataCollector {
    pub client: Client,
   pub config: CrawlConfig,
    pub pool: PgPool,
}

impl DataCollector {
    pub fn new(pool: PgPool, config: Option<CrawlConfig>) -> Self {
        let client = Client::builder()
            .user_agent("LotteryPredictionBot/1.0")
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .unwrap();
            
        Self {
            client,
            config: config.unwrap_or_default(),
            pool,
        }
    }

    pub async fn collect_all_data(&self) -> Result<Vec<LotteryDrawing>> {
        let mut all_data = Vec::new();
        
        for source in &self.config.sources {
            match self.collect_from_source(source).await {
                Ok(data) => all_data.extend(data),
                Err(e) => {
                    log::warn!("Failed to collect from {}: {}", source.name, e);
                    continue;
                }
            }
        }
        
        Ok(all_data)
    }

    async fn collect_from_source(&self, 
        source: &DataSource
    ) -> Result<Vec<LotteryDrawing>> {
        let mut retry_count = 0;
        
        loop {
            match timeout(
                std::time::Duration::from_secs(self.config.timeout_seconds),
                self.fetch_data(source)
            ).await {
                Ok(Ok(data)) => return Ok(data),
                Ok(Err(e)) | Err(_) => {
                    retry_count += 1;
                    if retry_count >= self.config.retry_attempts {
                        return Err(LotteryError::DataCollectionError(
                            format!("Failed to collect from {} after {} attempts", 
                                source.name, retry_count)
                        ));
                    }
                    sleep(std::time::Duration::from_millis(
                        self.config.retry_delay * retry_count as u64
                    )).await;
                }
            }
        }
    }

    async fn fetch_data(&self, 
        source: &DataSource
    ) -> Result<Vec<LotteryDrawing>> {
        let response = self.client.get(&source.url).send().await?;
        let body = response.text().await?;
        
        match source.parser {
            ParserType::Html => self.parse_html(&body, source),
            ParserType::Json => self.parse_json(&body, source),
            ParserType::Xml => self.parse_xml(&body, source),
            ParserType::Regex => self.parse_regex(&body, source),
        }
    }

    fn parse_html(
        &self, 
        html: &str, 
        source: &DataSource
    ) -> Result<Vec<LotteryDrawing>> {
        let document = Html::parse_document(html);
        let mut drawings = Vec::new();
        
        let table_selector = Selector::parse("table.tbl1 tr").unwrap();
        let rows = document.select(&table_selector);
        
        for row in rows.skip(1) { // Skip header
            let cells: Vec<_> = row.select(&Selector::parse("td").unwrap()).collect();
            
            if cells.len() >= 8 {
                let draw_number = cells[0].text().collect::<String>().trim().to_string();
                let draw_date = NaiveDate::parse_from_str(
                    &cells[1].text().collect::<String>().trim(),
                    "%Y-%m-%d"
                ).map_err(|_| LotteryError::DataCollectionError(
                    "Invalid date format".to_string()
                ))?;
                
                let numbers: Vec<u32> = cells[2..8]
                    .iter()
                    .map(|cell| cell.text().collect::<String>().trim().parse())
                    .collect::<Result<_, _>>()
                    .map_err(|_| LotteryError::DataCollectionError(
                        "Invalid number format".to_string()
                    ))?;
                
                let special_numbers = if cells.len() >= 9 {
                    Some(vec![cells[8].text().collect::<String>().trim().parse().unwrap()])
                } else {
                    None
                };
                
                let jackpot = cells.get(9)
                    .and_then(|c| c.text().collect::<String>().trim().parse().ok());
                
                drawings.push(LotteryDrawing {
                    id: Uuid::new_v4(),
                    lottery_type: source.lottery_type.clone(),
                    draw_number,
                    draw_date,
                    draw_time: None,
                    winning_numbers: numbers,
                    special_numbers,
                    jackpot_amount: jackpot,
                    sales_amount: None,
                    prize_distribution: None,
                    data_source: source.name.clone(),
                    verification_status: "pending".to_string(),
                    metadata: None,
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                    crawled_at: None,
                });
            }
        }
        
        Ok(drawings)
    }

    fn parse_json(
        &self, 
        json: &str, 
        source: &DataSource
    ) -> Result<Vec<LotteryDrawing>> {
        #[derive(Deserialize)]
        struct RawDrawing {
            #[serde(alias = "code", alias = "issue", alias = "period")]
            draw_number: String,
            #[serde(alias = "date", alias = "opendate")]
            draw_date: String,
            #[serde(alias = "red", alias = "redball", alias = "front")]
            red_balls: String,
            #[serde(alias = "blue", alias = "blueball", alias = "back")]
            blue_balls: Option<String>,
            #[serde(alias = "money", alias = "prize")]
            jackpot: Option<f64>,
        }
        
        let raw_data: Vec<RawDrawing> = serde_json::from_str(json)
            .map_err(|_| LotteryError::DataCollectionError(
                "Invalid JSON format".to_string()
            ))?;
        
        let mut drawings = Vec::new();
        
        for raw in raw_data {
            let draw_date = NaiveDate::parse_from_str(
                &raw.draw_date, "%Y-%m-%d"
            ).map_err(|_| LotteryError::DataCollectionError(
                "Invalid date format in JSON".to_string()
            ))?;
            
            let red_numbers: Vec<u32> = raw.red_balls
                .split(',')
                .filter_map(|s| s.trim().parse().ok())
                .collect();
            
            let blue_numbers = raw.blue_balls.map(|b| {
                b.split(',')
                    .filter_map(|s| s.trim().parse().ok())
                    .collect()
            });
            
            drawings.push(LotteryDrawing {
                id: Uuid::new_v4(),
                lottery_type: source.lottery_type.clone(),
                draw_number: raw.draw_number,
                draw_date,
                draw_time: None,
                winning_numbers: red_numbers,
                special_numbers: blue_numbers,
                jackpot_amount: raw.jackpot,
                sales_amount: None,
                prize_distribution: None,
                data_source: source.name.clone(),
                verification_status: "pending".to_string(),
                metadata: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
                crawled_at: None,
            });
        }
        
        Ok(drawings)
    }

    fn parse_xml(&self, xml: &str, source: &DataSource) -> Result<Vec<LotteryDrawing>> {
        use roxmltree::{Document, Node};
        
        let doc = Document::parse(xml)
            .map_err(|_| LotteryError::DataCollectionError(
                "Invalid XML format".to_string()
            ))?;
        
        let mut drawings = Vec::new();
        
        // 根据lottery_type选择不同的XML解析策略
        match source.lottery_type {
            LotteryType::Ssq => self.parse_xml_ssq(&doc)?,
            LotteryType::Dlt => self.parse_xml_dlt(&doc)?,
            LotteryType::Fc3d => self.parse_xml_fc3d(&doc)?,
            LotteryType::Pl3 => self.parse_xml_pl3(&doc)?,
            LotteryType::Pl5 => self.parse_xml_pl5(&doc)?,
            LotteryType::Custom => self.parse_xml_generic(&doc)?,
        }
        .into_iter()
        .for_each(|mut drawing| {
            drawing.lottery_type = source.lottery_type.clone();
            drawing.data_source = source.name.clone();
            drawings.push(drawing);
        });
        
        Ok(drawings)
    }
    
    fn parse_xml_ssq(&self, doc: &Document) -> Result<Vec<LotteryDrawing>> {
        let mut drawings = Vec::new();
        
        for record in doc.descendants().filter(|n| n.has_tag_name("record")) {
            let draw_number = self.get_xml_text(&record, "issue").unwrap_or_default();
            let draw_date = NaiveDate::parse_from_str(
                &self.get_xml_text(&record, "date").unwrap_or_default(),
                "%Y-%m-%d"
            ).map_err(|_| LotteryError::DataCollectionError(
                "Invalid date format in XML".to_string()
            ))?;
            
            let red_balls: Vec<u32> = self.get_xml_text(&record, "red")
                .unwrap_or_default()
                .split(',')
                .filter_map(|s| s.trim().parse().ok())
                .collect();
            
            let blue_ball = self.get_xml_text(&record, "blue")
                .unwrap_or_default()
                .parse()
                .ok();
            
            drawings.push(LotteryDrawing {
                id: Uuid::new_v4(),
                lottery_type: LotteryType::Ssq, // 临时值，会被外层覆盖
                draw_number,
                draw_date,
                draw_time: None,
                winning_numbers: red_balls,
                special_numbers: blue_ball.map(|b| vec![b]),
                jackpot_amount: self.get_xml_text(&record, "money")
                    .and_then(|s| s.parse().ok()),
                sales_amount: self.get_xml_text(&record, "sales")
                    .and_then(|s| s.parse().ok()),
                prize_distribution: None,
                data_source: String::new(), // 会被外层覆盖
                verification_status: "pending".to_string(),
                metadata: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
                crawled_at: None,
            });
        }
        
        Ok(drawings)
    }
    
    fn parse_xml_dlt(&self, doc: &Document) -> Result<Vec<LotteryDrawing>> {
        let mut drawings = Vec::new();
        
        for record in doc.descendants().filter(|n| n.has_tag_name("record")) {
            let draw_number = self.get_xml_text(&record, "issue").unwrap_or_default();
            let draw_date = NaiveDate::parse_from_str(
                &self.get_xml_text(&record, "date").unwrap_or_default(),
                "%Y-%m-%d"
            ).map_err(|_| LotteryError::DataCollectionError(
                "Invalid date format in XML".to_string()
            ))?;
            
            let front_balls: Vec<u32> = self.get_xml_text(&record, "front")
                .unwrap_or_default()
                .split(',')
                .filter_map(|s| s.trim().parse().ok())
                .collect();
            
            let back_balls: Vec<u32> = self.get_xml_text(&record, "back")
                .unwrap_or_default()
                .split(',')
                .filter_map(|s| s.trim().parse().ok())
                .collect();
            
            drawings.push(LotteryDrawing {
                id: Uuid::new_v4(),
                lottery_type: LotteryType::Dlt, // 临时值，会被外层覆盖
                draw_number,
                draw_date,
                draw_time: None,
                winning_numbers: front_balls,
                special_numbers: Some(back_balls),
                jackpot_amount: self.get_xml_text(&record, "money")
                    .and_then(|s| s.parse().ok()),
                sales_amount: self.get_xml_text(&record, "sales")
                    .and_then(|s| s.parse().ok()),
                prize_distribution: None,
                data_source: String::new(), // 会被外层覆盖
                verification_status: "pending".to_string(),
                metadata: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
                crawled_at: None,
            });
        }
        
        Ok(drawings)
    }
    
    fn parse_xml_fc3d(&self, doc: &Document) -> Result<Vec<LotteryDrawing>> {
        let mut drawings = Vec::new();
        
        for record in doc.descendants().filter(|n| n.has_tag_name("record")) {
            let draw_number = self.get_xml_text(&record, "issue").unwrap_or_default();
            let draw_date = NaiveDate::parse_from_str(
                &self.get_xml_text(&record, "date").unwrap_or_default(),
                "%Y-%m-%d"
            ).map_err(|_| LotteryError::DataCollectionError(
                "Invalid date format in XML".to_string()
            ))?;
            
            let numbers: Vec<u32> = self.get_xml_text(&record, "number")
                .unwrap_or_default()
                .chars()
                .filter_map(|c| c.to_digit(10))
                .collect();
            
            drawings.push(LotteryDrawing {
                id: Uuid::new_v4(),
                lottery_type: LotteryType::Fc3d, // 临时值，会被外层覆盖
                draw_number,
                draw_date,
                draw_time: None,
                winning_numbers: numbers,
                special_numbers: None,
                jackpot_amount: self.get_xml_text(&record, "money")
                    .and_then(|s| s.parse().ok()),
                sales_amount: self.get_xml_text(&record, "sales")
                    .and_then(|s| s.parse().ok()),
                prize_distribution: None,
                data_source: String::new(), // 会被外层覆盖
                verification_status: "pending".to_string(),
                metadata: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
                crawled_at: None,
            });
        }
        
        Ok(drawings)
    }
    
    fn parse_xml_pl3(&self, doc: &Document) -> Result<Vec<LotteryDrawing>> {
        self.parse_xml_fc3d(doc) // 排列3和福彩3D格式相似
    }
    
    fn parse_xml_pl5(&self, doc: &Document) -> Result<Vec<LotteryDrawing>> {
        let mut drawings = Vec::new();
        
        for record in doc.descendants().filter(|n| n.has_tag_name("record")) {
            let draw_number = self.get_xml_text(&record, "issue").unwrap_or_default();
            let draw_date = NaiveDate::parse_from_str(
                &self.get_xml_text(&record, "date").unwrap_or_default(),
                "%Y-%m-%d"
            ).map_err(|_| LotteryError::DataCollectionError(
                "Invalid date format in XML".to_string()
            ))?;
            
            let numbers: Vec<u32> = self.get_xml_text(&record, "number")
                .unwrap_or_default()
                .chars()
                .filter_map(|c| c.to_digit(10))
                .collect();
            
            drawings.push(LotteryDrawing {
                id: Uuid::new_v4(),
                lottery_type: LotteryType::Pl5, // 临时值，会被外层覆盖
                draw_number,
                draw_date,
                draw_time: None,
                winning_numbers: numbers,
                special_numbers: None,
                jackpot_amount: self.get_xml_text(&record, "money")
                    .and_then(|s| s.parse().ok()),
                sales_amount: self.get_xml_text(&record, "sales")
                    .and_then(|s| s.parse().ok()),
                prize_distribution: None,
                data_source: String::new(), // 会被外层覆盖
                verification_status: "pending".to_string(),
                metadata: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
                crawled_at: None,
            });
        }
        
        Ok(drawings)
    }
    
    fn parse_xml_generic(&self, doc: &Document) -> Result<Vec<LotteryDrawing>> {
        let mut drawings = Vec::new();
        
        // 通用XML解析，尝试多种可能的结构
        for record in doc.descendants().filter(|n| {
            n.has_tag_name("record") || n.has_tag_name("drawing") || n.has_tag_name("item")
        }) {
            let draw_number = self.get_xml_text(&record, "issue")
                .or_else(|| self.get_xml_text(&record, "number"))
                .or_else(|| self.get_xml_text(&record, "id"))
                .unwrap_or_default();
            
            let draw_date_str = self.get_xml_text(&record, "date")
                .or_else(|| self.get_xml_text(&record, "draw_date"))
                .or_else(|| self.get_xml_text(&record, "time"))
                .unwrap_or_default();
            
            let draw_date = NaiveDate::parse_from_str(&draw_date_str, "%Y-%m-%d")
                .or_else(|_| NaiveDate::parse_from_str(&draw_date_str, "%Y%m%d"))
                .map_err(|_| LotteryError::DataCollectionError(
                    "Invalid date format in XML".to_string()
                ))?;
            
            let numbers_str = self.get_xml_text(&record, "numbers")
                .or_else(|| self.get_xml_text(&record, "balls"))
                .or_else(|| self.get_xml_text(&record, "result"))
                .unwrap_or_default();
            
            let numbers: Vec<u32> = numbers_str
                .split(|c: char| !c.is_ascii_digit())
                .filter_map(|s| s.parse().ok())
                .collect();
            
            drawings.push(LotteryDrawing {
                id: Uuid::new_v4(),
                lottery_type: LotteryType::Ssq, // 临时值，会被外层覆盖
                draw_number,
                draw_date,
                draw_time: None,
                winning_numbers: numbers,
                special_numbers: None,
                jackpot_amount: None,
                sales_amount: None,
                prize_distribution: None,
                data_source: String::new(), // 会被外层覆盖
                verification_status: "pending".to_string(),
                metadata: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
                crawled_at: None,
            });
        }
        
        Ok(drawings)
    }
    
    fn get_xml_text(&self, node: &Node, tag_name: &str) -> Option<String> {
        node.children()
            .find(|n| n.has_tag_name(tag_name))
            .and_then(|n| n.text())
            .map(|s| s.trim().to_string())
    }

    fn parse_regex(&self, text: &str, source: &DataSource) -> Result<Vec<LotteryDrawing>> {
        let mut drawings = Vec::new();
        
        match source.lottery_type {
            LotteryType::Ssq => self.parse_regex_ssq(text)?,
            LotteryType::Dlt => self.parse_regex_dlt(text)?,
            LotteryType::Fc3d => self.parse_regex_fc3d(text)?,
            LotteryType::Pl3 => self.parse_regex_pl3(text)?,
            LotteryType::Pl5 => self.parse_regex_pl5(text)?,
            LotteryType::Custom(_) => self.parse_regex_generic(text)?,
        }
        .into_iter()
        .for_each(|mut drawing| {
            drawing.lottery_type = source.lottery_type.clone();
            drawing.data_source = source.name.clone();
            drawings.push(drawing);
        });
        
        Ok(drawings)
    }
    
    fn parse_regex_ssq(&self, text: &str) -> Result<Vec<LotteryDrawing>> {
        let mut drawings = Vec::new();
        
        // 双色球正则表达式模式
        let re = Regex::new(
            r"(?P<issue>\d{7})\s+(?P<date>\d{4}-\d{2}-\d{2})\s+(?P<red>[\d\s,]+)\s+(?P<blue>\d+)"|
            r"期号[:：\s]*(?P<issue>\d+)\s*日期[:：\s]*(?P<date>\d{4}-\d{2}-\d{2})\s*红球[:：\s]*(?P<red>[\d\s,]+)\s*蓝球[:：\s]*(?P<blue>\d+)"|
            r"(?P<issue>\d+)\s+(?P<date>\d{4}/\d{2}/\d{2})\s+(?P<red>\d{2}\s+\d{2}\s+\d{2}\s+\d{2}\s+\d{2}\s+\d{2})\s+(?P<blue>\d{2})"
        ).map_err(|_| LotteryError::DataCollectionError(
            "Invalid regex pattern".to_string()
        ))?;
        
        for cap in re.captures_iter(text) {
            let draw_number = cap.name("issue").map(|m| m.as_str().to_string()).unwrap_or_default();
            let draw_date_str = cap.name("date").map(|m| m.as_str()).unwrap_or_default();
            
            let draw_date = NaiveDate::parse_from_str(draw_date_str, "%Y-%m-%d")
                .or_else(|_| NaiveDate::parse_from_str(draw_date_str, "%Y/%m/%d"))
                .map_err(|_| LotteryError::DataCollectionError(
                    "Invalid date format in regex match".to_string()
                ))?;
            
            let red_str = cap.name("red").map(|m| m.as_str()).unwrap_or_default();
            let red_numbers: Vec<u32> = red_str
                .split(|c: char| !c.is_ascii_digit())
                .filter_map(|s| s.parse().ok())
                .collect();
            
            let blue_str = cap.name("blue").map(|m| m.as_str()).unwrap_or_default();
            let blue_number = blue_str.parse().ok();
            
            drawings.push(LotteryDrawing {
                id: Uuid::new_v4(),
                lottery_type: LotteryType::Ssq, // 临时值，会被外层覆盖
                draw_number,
                draw_date,
                draw_time: None,
                winning_numbers: red_numbers,
                special_numbers: blue_number.map(|b| vec![b]),
                jackpot_amount: None,
                sales_amount: None,
                prize_distribution: None,
                data_source: String::new(), // 会被外层覆盖
                verification_status: "pending".to_string(),
                metadata: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
                crawled_at: None,
            });
        }
        
        Ok(drawings)
    }
    
    fn parse_regex_dlt(&self, text: &str) -> Result<Vec<LotteryDrawing>> {
        let mut drawings = Vec::new();
        
        // 大乐透正则表达式模式
        let re = Regex::new(
            r"(?P<issue>\d{5})\s+(?P<date>\d{4}-\d{2}-\d{2})\s+(?P<front>[\d\s,]+)\s+(?P<back>[\d\s,]+)"|
            r"期号[:：\s]*(?P<issue>\d+)\s*日期[:：\s]*(?P<date>\d{4}-\d{2}-\d{2})\s*前区[:：\s]*(?P<front>[\d\s,]+)\s*后区[:：\s]*(?P<back>[\d\s,]+)"
        ).map_err(|_| LotteryError::DataCollectionError(
            "Invalid regex pattern".to_string()
        ))?;
        
        for cap in re.captures_iter(text) {
            let draw_number = cap.name("issue").map(|m| m.as_str().to_string()).unwrap_or_default();
            let draw_date_str = cap.name("date").map(|m| m.as_str()).unwrap_or_default();
            
            let draw_date = NaiveDate::parse_from_str(draw_date_str, "%Y-%m-%d")
                .or_else(|_| NaiveDate::parse_from_str(draw_date_str, "%Y/%m/%d"))
                .map_err(|_| LotteryError::DataCollectionError(
                    "Invalid date format in regex match".to_string()
                ))?;
            
            let front_str = cap.name("front").map(|m| m.as_str()).unwrap_or_default();
            let front_numbers: Vec<u32> = front_str
                .split(|c: char| !c.is_ascii_digit())
                .filter_map(|s| s.parse().ok())
                .collect();
            
            let back_str = cap.name("back").map(|m| m.as_str()).unwrap_or_default();
            let back_numbers: Vec<u32> = back_str
                .split(|c: char| !c.is_ascii_digit())
                .filter_map(|s| s.parse().ok())
                .collect();
            
            drawings.push(LotteryDrawing {
                id: Uuid::new_v4(),
                lottery_type: LotteryType::Dlt, // 临时值，会被外层覆盖
                draw_number,
                draw_date,
                draw_time: None,
                winning_numbers: front_numbers,
                special_numbers: Some(back_numbers),
                jackpot_amount: None,
                sales_amount: None,
                prize_distribution: None,
                data_source: String::new(), // 会被外层覆盖
                verification_status: "pending".to_string(),
                metadata: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
                crawled_at: None,
            });
        }
        
        Ok(drawings)
    }
    
    fn parse_regex_fc3d(&self, text: &str) -> Result<Vec<LotteryDrawing>> {
        let mut drawings = Vec::new();
        
        // 福彩3D正则表达式模式
        let re = Regex::new(
            r"(?P<issue>\d{3})\s+(?P<date>\d{4}-\d{2}-\d{2})\s+(?P<number>\d{3})"|
            r"(?P<date>\d{4}-\d{2}-\d{2})\s+(?P<issue>\d+)\s+(?P<number>\d{3})"|
            r"期号[:：\s]*(?P<issue>\d+)\s*日期[:：\s]*(?P<date>\d{4}-\d{2}-\d{2})\s*开奖号码[:：\s]*(?P<number>\d{3})"
        ).map_err(|_| LotteryError::DataCollectionError(
            "Invalid regex pattern".to_string()
        ))?;
        
        for cap in re.captures_iter(text) {
            let draw_number = cap.name("issue").map(|m| m.as_str().to_string()).unwrap_or_default();
            let draw_date_str = cap.name("date").map(|m| m.as_str()).unwrap_or_default();
            
            let draw_date = NaiveDate::parse_from_str(draw_date_str, "%Y-%m-%d")
                .or_else(|_| NaiveDate::parse_from_str(draw_date_str, "%Y/%m/%d"))
                .map_err(|_| LotteryError::DataCollectionError(
                    "Invalid date format in regex match".to_string()
                ))?;
            
            let number_str = cap.name("number").map(|m| m.as_str()).unwrap_or_default();
            let numbers: Vec<u32> = number_str
                .chars()
                .filter_map(|c| c.to_digit(10))
                .collect();
            
            drawings.push(LotteryDrawing {
                id: Uuid::new_v4(),
                lottery_type: LotteryType::Fc3d, // 临时值，会被外层覆盖
                draw_number,
                draw_date,
                draw_time: None,
                winning_numbers: numbers,
                special_numbers: None,
                jackpot_amount: None,
                sales_amount: None,
                prize_distribution: None,
                data_source: String::new(), // 会被外层覆盖
                verification_status: "pending".to_string(),
                metadata: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
                crawled_at: None,
            });
        }
        
        Ok(drawings)
    }
    
    fn parse_regex_pl3(&self, text: &str) -> Result<Vec<LotteryDrawing>> {
        self.parse_regex_fc3d(text) // 排列3和福彩3D格式相似
    }
    
    fn parse_regex_pl5(&self, text: &str) -> Result<Vec<LotteryDrawing>> {
        let mut drawings = Vec::new();
        
        // 排列5正则表达式模式
        let re = Regex::new(
            r"(?P<issue>\d{5})\s+(?P<date>\d{4}-\d{2}-\d{2})\s+(?P<number>\d{5})"|
            r"(?P<date>\d{4}-\d{2}-\d{2})\s+(?P<issue>\d+)\s+(?P<number>\d{5})"|
            r"期号[:：\s]*(?P<issue>\d+)\s*日期[:：\s]*(?P<date>\d{4}-\d{2}-\d{2})\s*开奖号码[:：\s]*(?P<number>\d{5})"
        ).map_err(|_| LotteryError::DataCollectionError(
            "Invalid regex pattern".to_string()
        ))?;
        
        for cap in re.captures_iter(text) {
            let draw_number = cap.name("issue").map(|m| m.as_str().to_string()).unwrap_or_default();
            let draw_date_str = cap.name("date").map(|m| m.as_str()).unwrap_or_default();
            
            let draw_date = NaiveDate::parse_from_str(draw_date_str, "%Y-%m-%d")
                .or_else(|_| NaiveDate::parse_from_str(draw_date_str, "%Y/%m/%d"))
                .map_err(|_| LotteryError::DataCollectionError(
                    "Invalid date format in regex match".to_string()
                ))?;
            
            let number_str = cap.name("number").map(|m| m.as_str()).unwrap_or_default();
            let numbers: Vec<u32> = number_str
                .chars()
                .filter_map(|c| c.to_digit(10))
                .collect();
            
            drawings.push(LotteryDrawing {
                id: Uuid::new_v4(),
                lottery_type: LotteryType::Pl5, // 临时值，会被外层覆盖
                draw_number,
                draw_date,
                draw_time: None,
                winning_numbers: numbers,
                special_numbers: None,
                jackpot_amount: None,
                sales_amount: None,
                prize_distribution: None,
                data_source: String::new(), // 会被外层覆盖
                verification_status: "pending".to_string(),
                metadata: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
                crawled_at: None,
            });
        }
        
        Ok(drawings)
    }
    
    fn parse_regex_generic(&self, text: &str) -> Result<Vec<LotteryDrawing>> {
        let mut drawings = Vec::new();
        
        // 通用正则表达式模式，尝试提取数字序列
        let re = Regex::new(
            r"(?P<issue>\d+)\s+(?P<date>\d{4}-\d{2}-\d{2})\s+(?P<numbers>[\d\s,]+)"
        ).map_err(|_| LotteryError::DataCollectionError(
            "Invalid regex pattern".to_string()
        ))?;
        
        for cap in re.captures_iter(text) {
            let draw_number = cap.name("issue").map(|m| m.as_str().to_string()).unwrap_or_default();
            let draw_date_str = cap.name("date").map(|m| m.as_str()).unwrap_or_default();
            
            let draw_date = NaiveDate::parse_from_str(draw_date_str, "%Y-%m-%d")
                .or_else(|_| NaiveDate::parse_from_str(draw_date_str, "%Y/%m/%d"))
                .map_err(|_| LotteryError::DataCollectionError(
                    "Invalid date format in regex match".to_string()
                ))?;
            
            let numbers_str = cap.name("numbers").map(|m| m.as_str()).unwrap_or_default();
            let numbers: Vec<u32> = numbers_str
                .split(|c: char| !c.is_ascii_digit())
                .filter_map(|s| s.parse().ok())
                .collect();
            
            drawings.push(LotteryDrawing {
                id: Uuid::new_v4(),
                lottery_type: LotteryType::Ssq, // 临时值，会被外层覆盖
                draw_number,
                draw_date,
                draw_time: None,
                winning_numbers: numbers,
                special_numbers: None,
                jackpot_amount: None,
                sales_amount: None,
                prize_distribution: None,
                data_source: String::new(), // 会被外层覆盖
                verification_status: "pending".to_string(),
                metadata: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
                crawled_at: None,
            });
        }
        
        Ok(drawings)
    }

    pub async fn save_data(&self, 
        drawings: Vec<LotteryDrawing>
    ) -> Result<usize> {
        let mut saved_count = 0;
        
        for chunk in drawings.chunks(self.config.batch_size) {
            let mut query_builder = sqlx::QueryBuilder::new(
                r#"
                INSERT INTO lottery_drawings (
                    id, lottery_type_id, draw_number, draw_date, draw_time,
                    winning_numbers, special_numbers, jackpot_amount, sales_amount,
                    prize_distribution, data_source, verification_status, metadata,
                    created_at, updated_at, crawled_at
                )
                SELECT 
                    d.id,
                    lt.id,
                    d.draw_number,
                    d.draw_date,
                    d.draw_time,
                    d.winning_numbers,
                    d.special_numbers,
                    d.jackpot_amount,
                    d.sales_amount,
                    d.prize_distribution,
                    d.data_source,
                    d.verification_status,
                    d.metadata,
                    d.created_at,
                    d.updated_at,
                    d.crawled_at
                FROM (
                "#
            );
            
            query_builder.push_values(chunk, |mut b, drawing| {
                b.push_bind(drawing.id)
                 .push_bind(&drawing.lottery_type.to_string())
                 .push_bind(&drawing.draw_number)
                 .push_bind(drawing.draw_date)
                 .push_bind(drawing.draw_time)
                 .push_bind(&drawing.winning_numbers)
                 .push_bind(&drawing.special_numbers)
                 .push_bind(drawing.jackpot_amount)
                 .push_bind(drawing.sales_amount)
                 .push_bind(&drawing.prize_distribution)
                 .push_bind(&drawing.data_source)
                 .push_bind(&drawing.verification_status)
                 .push_bind(&drawing.metadata)
                 .push_bind(drawing.created_at)
                 .push_bind(drawing.updated_at)
                 .push_bind(drawing.crawled_at);
            });
            
            query_builder.push(
                r#"
                ) d
                CROSS JOIN LATERAL (
                    SELECT id FROM lottery_types WHERE name = d.lottery_type
                ) lt
                ON CONFLICT (lottery_type_id, draw_number) 
                DO UPDATE SET
                    updated_at = EXCLUDED.updated_at,
                    verification_status = EXCLUDED.verification_status,
                    winning_numbers = EXCLUDED.winning_numbers,
                    special_numbers = EXCLUDED.special_numbers,
                    jackpot_amount = EXCLUDED.jackpot_amount,
                    sales_amount = EXCLUDED.sales_amount,
                    crawled_at = EXCLUDED.crawled_at
                "#
            );
            
            let result = query_builder.build().execute(&self.pool).await?;
            saved_count += result.rows_affected() as usize;
        }
        
        Ok(saved_count)
    }

    pub async fn validate_data(&self, 
        drawings: &[LotteryDrawing]
    ) -> Result<Vec<LotteryDrawing>> {
        let mut validated = Vec::new();
        
        for drawing in drawings {
            let is_valid = self.validate_single_drawing(drawing).await?;
            if is_valid {
                validated.push(drawing.clone());
            }
        }
        
        Ok(validated)
    }

    async fn validate_single_drawing(
        &self, 
        drawing: &LotteryDrawing
    ) -> Result<bool> {
        // 检查日期合理性
        if drawing.draw_date > Utc::now().date_naive() {
            return Ok(false);
        }
        
        // 检查号码范围
        for &num in &drawing.winning_numbers {
            if num < 1 || num > 50 { // 根据彩票类型调整
                return Ok(false);
            }
        }
        
        // 检查重复号码
        let mut seen = std::collections::HashSet::new();
        for &num in &drawing.winning_numbers {
            if !seen.insert(num) {
                return Ok(false);
            }
        }
        
        Ok(true)
    }

    pub async fn get_missing_dates(
        &self,
        lottery_type: &LotteryType,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<Vec<NaiveDate>> {
        let existing_dates: Vec<NaiveDate> = sqlx::query_scalar(
            r#"
            SELECT draw_date FROM lottery_drawings
            WHERE lottery_type_id = $1 
            AND draw_date BETWEEN $2 AND $3
            ORDER BY draw_date
            "#
        )
        .bind(lottery_type.to_string())
        .bind(start_date)
        .bind(end_date)
        .fetch_all(&self.pool)
        .await?;
        
        let mut missing_dates = Vec::new();
        let mut current_date = start_date;
        
        while current_date <= end_date {
            if !existing_dates.contains(&current_date) {
                missing_dates.push(current_date);
            }
            current_date += Duration::days(1);
        }
        
        Ok(missing_dates)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Days;

    #[tokio::test]
    async fn test_data_collector_creation() {
        let config = CrawlConfig::default();
        assert!(!config.sources.is_empty());
        assert_eq!(config.retry_attempts, 3);
    }

    #[tokio::test]
    async fn test_validate_drawing() {
        let collector = DataCollector::new(PgPool::connect("postgres://localhost/test").await.unwrap(), None);
        
        let valid_drawing = LotteryDrawing {
            id: Uuid::new_v4(),
            lottery_type: LotteryType::Ssq,
            draw_number: "2024001".to_string(),
            draw_date: Utc::now().date_naive() - Days::new(1),
            draw_time: None,
            winning_numbers: vec![1, 2, 3, 4, 5, 6],
            special_numbers: Some(vec![7]),
            jackpot_amount: Some(1000000.0),
            sales_amount: None,
            prize_distribution: None,
            data_source: "test".to_string(),
            verification_status: "pending".to_string(),
            metadata: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            crawled_at: None,
        };
        
        let result = collector.validate_single_drawing(&valid_drawing).await;
        assert!(result.is_ok());
    }
}