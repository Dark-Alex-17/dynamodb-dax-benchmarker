use anyhow::anyhow;
use aws_sdk_dynamodb::types::AttributeValue;
use log::{error, info};

use crate::{
  models::{BenchmarkingItem, DynamoDbSimulationMetrics},
  time,
};

use super::{utils::extract_partition_key, Simulator};

impl<'a> Simulator<'a> {
  pub async fn read_item(
    &mut self,
    id: AttributeValue,
    metrics: &mut DynamoDbSimulationMetrics,
    record_metrics: bool,
  ) -> anyhow::Result<Option<BenchmarkingItem>> {
    let partition_key = extract_partition_key(id.clone());
    let (read_time, response) = time!(
      resp,
      self
        .dynamodb_client
        .get_item()
        .table_name(self.table_name.clone())
        .key("id", id)
        .send()
        .await
    );

    if record_metrics {
      metrics.read_time = Some(read_time);
    }

    match response {
      Ok(resp) => {
        info!("Found item: {}", partition_key);
        if let Some(item) = resp.item() {
          info!("Fetched item: {item:?}");
          Ok(Some(BenchmarkingItem::from(item.clone())))
        } else {
          info!("No items found with partition key: {partition_key}");
          Ok(None)
        }
      }
      Err(e) => {
        error!("Could not fetch item with partition key: {partition_key}. {e:?}");
        Err(anyhow!(e))
      }
    }
  }

  pub async fn update_item(
    &mut self,
    id: AttributeValue,
    metrics: &mut DynamoDbSimulationMetrics,
  ) -> anyhow::Result<()> {
    let mut updated_item = BenchmarkingItem::new(self.attributes);
    updated_item.insert("id", id.clone());
    let partition_key = extract_partition_key(id);
    let (update_time, response) = time!(
      resp,
      self
        .dynamodb_client
        .put_item()
        .table_name(self.table_name.clone())
        .set_item(Some(updated_item.extract_map()))
        .send()
        .await
    );
    metrics.update_time = Some(update_time);

    match response {
      Ok(_) => {
        info!("Successfully updated item with partition_key: {partition_key}");
        Ok(())
      }
      Err(e) => {
        error!("Could not update item with partition key: {partition_key}. {e:?}");
        Err(anyhow!(e))
      }
    }
  }

  pub async fn put_item(
    &mut self,
    metrics: &mut DynamoDbSimulationMetrics,
  ) -> anyhow::Result<BenchmarkingItem> {
    let new_item = BenchmarkingItem::new(self.attributes);
    let partition_key = extract_partition_key(new_item.get("id").cloned().unwrap());
    let (time, response) = time!(
      resp,
      self
        .dynamodb_client
        .put_item()
        .table_name(self.table_name.clone())
        .set_item(Some(new_item.extract_map()))
        .send()
        .await
    );
    metrics.write_time = Some(time);

    match response {
      Ok(_) => {
        info!("Successfully put new item with partition key: {partition_key}");
        Ok(new_item)
      }
      Err(e) => {
        error!("Could not put new item with partition key: {partition_key}. {e:?}");
        Err(anyhow!(e))
      }
    }
  }

  pub async fn delete_item(
    &mut self,
    id: AttributeValue,
    metrics: &mut DynamoDbSimulationMetrics,
  ) -> anyhow::Result<()> {
    let partition_key = extract_partition_key(id.clone());
    let (delete_time, response) = time!(
      resp,
      self
        .dynamodb_client
        .delete_item()
        .table_name(self.table_name.clone())
        .key("id", id)
        .send()
        .await
    );
    metrics.delete_time = Some(delete_time);

    match response {
      Ok(_) => {
        info!("Successfully deleted item with partition key: {partition_key}");
        Ok(())
      }
      Err(e) => {
        error!("Could not delete item with partition key: {partition_key}. {e:?}");
        Err(anyhow!(e))
      }
    }
  }
}
