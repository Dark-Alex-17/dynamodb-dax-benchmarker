use aws_sdk_dynamodb::types::AttributeValue;

pub(super) fn extract_partition_key(id: AttributeValue) -> String {
  id.clone().as_s().unwrap().to_string()
}
