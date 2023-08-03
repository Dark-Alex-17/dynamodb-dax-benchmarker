package simulators

import (
	"time"

	"github.com/Dark-Alex-17/dynamodb-dax-benchmarker/pkg/models"
	"github.com/aws/aws-dax-go/dax"
	"github.com/aws/aws-sdk-go/service/dynamodb"
	log "github.com/sirupsen/logrus"
)

func ReadItem(client *dax.Dax, tableName string, id dynamodb.AttributeValue, metrics *models.DynamoDbSimulationMetrics, recordMetrics bool) (dynamodb.GetItemOutput, error) {
	partitionKey := *id.S
	startTime := time.Now()
	response, err := client.GetItem(&dynamodb.GetItemInput{
		TableName: &tableName,
		Key: map[string]*dynamodb.AttributeValue{
			"id": {S: id.S},
		},
	})

	if recordMetrics {
		duration := time.Since(startTime).Microseconds()
		millisecondDuration := float64(duration) / 1000
		metrics.ReadTime = &millisecondDuration
	}

	if err != nil {
		log.Errorf("Could not fetch item with partition key: %v. %v+", partitionKey, err)
		metrics.Successful = false
		return dynamodb.GetItemOutput{}, err
	}

	if len(response.Item) == 0 {
		log.Infof("No items found with partition key: %v", partitionKey)
		return dynamodb.GetItemOutput{}, nil
	}

	return *response, nil
}

func UpdateItem(client *dax.Dax, tableName string, id dynamodb.AttributeValue, attributes int, metrics *models.DynamoDbSimulationMetrics) {
	updatedItem := models.NewBenchmarkingItem(attributes)
	updatedItem["id"] = &id
	partitionKey := *id.S
	startTime := time.Now()

	_, err := client.PutItem(&dynamodb.PutItemInput{
		TableName: &tableName,
		Item:      updatedItem,
	})

	duration := time.Since(startTime).Microseconds()
	millisecondDuration := float64(duration) / 1000
	metrics.UpdateTime = &millisecondDuration

	if err != nil {
		log.Errorf("Could not update item with partition key: %v. %v+", partitionKey, err)
		metrics.Successful = false
	} else {
		log.Infof("Successfully updated item with partition key: %v", partitionKey)
	}
}

func PutItem(client *dax.Dax, tableName string, attributes int, metrics *models.DynamoDbSimulationMetrics) (models.BenchmarkingItem, error) {
	newItem := models.NewBenchmarkingItem(attributes)
	partitionKey := *newItem["id"].S
	startTime := time.Now()

	_, err := client.PutItem(&dynamodb.PutItemInput{
		TableName: &tableName,
		Item:      newItem,
	})

	duration := time.Since(startTime).Microseconds()
	millisecondDuration := float64(duration) / 1000
	metrics.WriteTime = &millisecondDuration

	if err != nil {
		log.Errorf("Could not put new item with partition key: %v. %v+", partitionKey, err)
		metrics.Successful = false
		return models.BenchmarkingItem{}, err
	}

	log.Infof("Successfully put new item with partition key: %v", partitionKey)
	return newItem, nil
}

func DeleteItem(client *dax.Dax, tableName string, id dynamodb.AttributeValue, metrics *models.DynamoDbSimulationMetrics) {
	partitionKey := *id.S
	startTime := time.Now()

	_, err := client.DeleteItem(&dynamodb.DeleteItemInput{
		TableName: &tableName,
		Key: map[string]*dynamodb.AttributeValue{
			"charger_id": &id,
		},
	})

	duration := time.Since(startTime).Microseconds()
	millisecondDuration := float64(duration) / 1000
	metrics.DeleteTime = &millisecondDuration

	if err != nil {
		log.Errorf("Could not delete item with partition key: %v. %v+", partitionKey, err)
		metrics.Successful = false
	} else {
		log.Infof("Successfully deleted item with partition key: %v", partitionKey)
	}
}
