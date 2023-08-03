use std::collections::HashMap;

use aws_sdk_dynamodb::types::AttributeValue;
use chrono::{DateTime, Utc};
use rand::Rng;
use serde::Serialize;
use serde_json::Number;
use uuid::Uuid;

#[derive(Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub enum DynamoOperation {
  #[default]
  Read,
  Write,
  Update,
}

impl From<i32> for DynamoOperation {
  fn from(value: i32) -> Self {
    match value {
      0 => DynamoOperation::Read,
      1 => DynamoOperation::Write,
      2 => DynamoOperation::Update,
      _ => DynamoOperation::Read,
    }
  }
}

#[derive(Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub enum Scenario {
  #[default]
  Crud,
  ReadOnly,
}

#[derive(Debug)]
pub struct BenchmarkingItem(HashMap<String, AttributeValue>);

impl From<HashMap<String, AttributeValue>> for BenchmarkingItem {
  fn from(value: HashMap<String, AttributeValue>) -> BenchmarkingItem {
    BenchmarkingItem(value)
  }
}

impl BenchmarkingItem {
  pub fn new(attributes: u32) -> BenchmarkingItem {
    let mut benchmarking_item = HashMap::<String, AttributeValue>::new();
    let mut rng = rand::thread_rng();
    benchmarking_item.insert(
      "id".to_owned(),
      AttributeValue::S(Uuid::new_v4().to_string()),
    );

    (0..attributes).for_each(|i| {
      if let 0 = i % 2 {
        benchmarking_item.insert(i.to_string(), AttributeValue::S(lipsum::lipsum_words(15)));
      } else {
        benchmarking_item.insert(
          i.to_string(),
          AttributeValue::N(rng.gen_range(0.0..=32.0).to_string()),
        );
      }
    });

    BenchmarkingItem(benchmarking_item)
  }

  pub fn get_id(&self) -> AttributeValue {
    self.0.get("id").cloned().unwrap()
  }

  pub fn insert(&mut self, key: &str, val: AttributeValue) -> Option<AttributeValue> {
    self.0.insert(key.to_owned(), val)
  }

  pub(crate) fn get(&self, key: &str) -> Option<&AttributeValue> {
    self.0.get(key)
  }

  pub fn extract_map(&self) -> HashMap<String, AttributeValue> {
    self.0.clone()
  }
}

#[derive(Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct DynamoDbSimulationMetrics {
  pub operation: DynamoOperation,
  pub timestamp: DateTime<Utc>,
  pub successful: bool,
  pub scenario: Scenario,
  pub simulation_time: Option<Number>,
  pub read_time: Option<Number>,
  pub write_time: Option<Number>,
  pub write_item_confirmation_time: Option<Number>,
  pub update_time: Option<Number>,
  pub update_item_confirmation_time: Option<Number>,
  pub delete_time: Option<Number>,
  pub delete_item_confirmation_time: Option<Number>,
}
