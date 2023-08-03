use aws_sdk_dynamodb::{types::AttributeValue, Client};
use log::{error, info};
use rand::{
  rngs::{OsRng, StdRng},
  Rng, SeedableRng,
};

use crate::{models::DynamoDbSimulationMetrics, time};

mod assertions;
mod operations;
mod utils;

pub struct Simulator<'a> {
  dynamodb_client: &'a Client,
  table_name: String,
  attributes: u32,
  partition_keys_vec: &'a [String],
  rng: StdRng,
}

impl<'a> Simulator<'a> {
  pub fn new(
    dynamodb_client: &'a Client,
    table_name: String,
    attributes: u32,
    partition_keys_vec: &'a [String],
  ) -> Simulator<'a> {
    Simulator {
      dynamodb_client,
      table_name,
      attributes,
      partition_keys_vec,
      rng: StdRng::from_seed(OsRng.gen()),
    }
  }

  pub async fn simulate_read_operation(
    &mut self,
    metrics: &mut DynamoDbSimulationMetrics,
  ) -> anyhow::Result<()> {
    info!("Performing READ Operation...");
    let partition_key =
      self.partition_keys_vec[self.rng.gen_range(0..self.partition_keys_vec.len())].clone();
    let id = AttributeValue::S(partition_key.clone());

    for i in 0..10 {
      info!("Attempt {i}: Fetching existing item with partition key: {partition_key}");

      match self.read_item(id.clone(), metrics, true).await? {
        Some(_) => {
          info!("Successfully read existing item with partition key: {partition_key}");
          break;
        }
        None => {
          error!("Unable to find existing item with partition key: {partition_key}");
          if i == 9 {
            error!(
              "All attempts to fetch the existing item with partition key: {partition_key} failed!"
            );
          }
        }
      }
    }

    Ok(())
  }

  pub async fn simulate_write_operation(
    &mut self,
    metrics: &mut DynamoDbSimulationMetrics,
  ) -> anyhow::Result<()> {
    info!("Performing WRITE operation...");
    let benchmarking_item = self.put_item(metrics).await?;
    let id = benchmarking_item.get_id();

    self.assert_item_was_created(id.clone(), metrics).await?;

    self.delete_item(id.clone(), metrics).await?;

    self.assert_item_was_deleted(id, metrics).await?;

    Ok(())
  }

  pub async fn simulate_update_operation(
    &mut self,
    metrics: &mut DynamoDbSimulationMetrics,
  ) -> anyhow::Result<()> {
    info!("Performing UPDATE operation...");
    let new_item = self.put_item(metrics).await?;
    let id = new_item.get_id();
    let partition_key = utils::extract_partition_key(id.clone());
    let mut attempts_exhausted = false;

    self.assert_item_was_created(id.clone(), metrics).await?;
    self.update_item(id.clone(), metrics).await?;

    let update_confirmation_time = time!(for i in 0..10 {
      info!("Attempt {i}: Fetching updated item for partition key: {partition_key}...");

      let updated_item = self.read_item(id.clone(), metrics, false).await?.unwrap();

      let new_item_attribute_value = new_item
        .get("1")
        .cloned()
        .unwrap()
        .as_n()
        .unwrap()
        .to_string();
      let updated_item_attribute_value = updated_item
        .get("1")
        .cloned()
        .unwrap()
        .as_n()
        .unwrap()
        .to_string();

      if new_item_attribute_value != updated_item_attribute_value {
        info!("Confirmed update for partition key: {partition_key}");
        break;
      } else {
        error!("Update for partition key {partition_key} failed! Values are still equal!");
        if i == 9 {
          error!("Exhausted attempts to fetch updated item!");
          attempts_exhausted = true;
        }
      }
    });

    if !attempts_exhausted {
      metrics.update_item_confirmation_time = Some(update_confirmation_time);
    }

    self.delete_item(id.clone(), metrics).await?;
    self.assert_item_was_deleted(id, metrics).await?;

    Ok(())
  }
}
