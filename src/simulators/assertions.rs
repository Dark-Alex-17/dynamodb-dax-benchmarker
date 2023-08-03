use aws_sdk_dynamodb::types::AttributeValue;
use log::{error, info};

use crate::{models::DynamoDbSimulationMetrics, time};

use super::{utils, Simulator};

impl<'a> Simulator<'a> {
  pub(super) async fn assert_item_was_created(
    &mut self,
    id: AttributeValue,
    metrics: &mut DynamoDbSimulationMetrics,
  ) -> anyhow::Result<()> {
    let partition_key = utils::extract_partition_key(id.clone());
    let mut attempts_exhausted = false;

    let write_confirmation_time = time!(for i in 0..10 {
      info!("Attempt {i}: Fetching newly added item with partition key: {partition_key}");

      match self.read_item(id.clone(), metrics, false).await? {
        Some(_) => {
          info!("Successfully read new item with partition key: {partition_key}");
          break;
        }
        None => {
          error!("Unable to find new item with partition key: {partition_key}");
          if i == 9 {
            error!("All attempts to fetch the newly added item with partition key: {partition_key} failed!");
            attempts_exhausted = true;
          }
        }
      };
    });

    if !attempts_exhausted {
      metrics.write_item_confirmation_time = Some(write_confirmation_time);
    }

    Ok(())
  }

  pub(super) async fn assert_item_was_deleted(
    &mut self,
    id: AttributeValue,
    metrics: &mut DynamoDbSimulationMetrics,
  ) -> anyhow::Result<()> {
    let partition_key = utils::extract_partition_key(id.clone());
    let mut attempts_exhausted = false;
    let delete_confirmation_time = time!(for i in 0..10 {
      info!("Attempt {i}: Fetching deleted item with partition key: {partition_key}...");
      match self.read_item(id.clone(), metrics, false).await? {
        Some(_) => {
          error!("Item with partition key {partition_key} was not deleted as expected!");
          if i == 9 {
            error!("All attempts to receive an empty response to verify item with partition key: {partition_key} was deleted failed!");
            attempts_exhausted = true;
          }
        }
        None => {
          info!("Item with partition key {partition_key} was successfully deleted.");
          break;
        }
      }
    });

    if !attempts_exhausted {
      metrics.delete_item_confirmation_time = Some(delete_confirmation_time);
    }

    Ok(())
  }
}
