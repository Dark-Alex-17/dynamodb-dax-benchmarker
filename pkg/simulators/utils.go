package simulators

import (
	"time"

	"github.com/Dark-Alex-17/dynamodb-dax-benchmarker/pkg/models"
	"github.com/aws/aws-dax-go/dax"
	"github.com/aws/aws-sdk-go/service/dynamodb"
	log "github.com/sirupsen/logrus"
)

func AssertItemWasCreated(client *dax.Dax, tableName string, id dynamodb.AttributeValue, metrics *models.DynamoDbSimulationMetrics) {
	partitionKey := *id.S
	attemptsExhausted := false
	startTime := time.Now()

	for i := 0; i < 10; i++ {
		log.Infof("Attempt %d: Fetching newly added item with partition key: %v", i, partitionKey)

		newItem, err := ReadItem(client, tableName, id, metrics, false)

		if err != nil || newItem.Item["id"].S == nil {
			log.Errorf("Unable to find new item with partition key: %v", partitionKey)
			if i == 9 {
				log.Errorf("All attempts to fetch the newly added item with partition key: %v failed!", partitionKey)
				attemptsExhausted = true
				metrics.Successful = false
			}
		} else {
			log.Infof("Successfully read new item with partition key: %v", partitionKey)
			break
		}
	}

	if !attemptsExhausted {
		duration := time.Since(startTime).Microseconds()
		millisecondDuration := float64(duration) / 1000
		metrics.WriteItemConfirmationTime = &millisecondDuration
	}
}

func AssertItemWasDeleted(client *dax.Dax, tableName string, id dynamodb.AttributeValue, metrics *models.DynamoDbSimulationMetrics) {
	partitionKey := *id.S
	attemptsExhausted := false
	startTime := time.Now()

	for i := 0; i < 10; i++ {
		log.Infof("Attempt %d: Fetching deleted item with partition key: %v ...", i, partitionKey)

		deletedItem, _ := ReadItem(client, tableName, id, metrics, false)
		if deletedItem.Item["id"].S == nil {
			log.Infof("Item with partition key: %v was successfully deleted.", partitionKey)
			break
		} else {
			log.Errorf("Item with partition key %v was not deleted as expected!", partitionKey)
			if i == 9 {
				log.Errorf("All attempts to receive an empty response to verify item with partition key: %v was deleted failed!", partitionKey)
				attemptsExhausted = true
				metrics.Successful = false
			}
		}
	}

	if !attemptsExhausted {
		duration := time.Since(startTime).Microseconds()
		millisecondDuration := float64(duration) / 1000
		metrics.DeleteItemConfirmationTime = &millisecondDuration
	}
}
