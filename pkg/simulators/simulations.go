package simulators

import (
	"math/rand"
	"strings"
	"time"

	"github.com/Dark-Alex-17/dynamodb-dax-benchmarker/pkg/models"
	"github.com/aws/aws-dax-go/dax"
	"github.com/aws/aws-sdk-go/service/dynamodb"
	log "github.com/sirupsen/logrus"
)

func SimulateReadOperation(client *dax.Dax, tableName string, partitionKeys []string, metrics *models.DynamoDbSimulationMetrics) {
	log.Info("Performing READ operation...")
	r := rand.New(rand.NewSource(time.Now().UnixNano()))
	var partitionKey string
	for {
		partitionKey = partitionKeys[r.Intn(len(partitionKeys))]
		if len(strings.TrimSpace(partitionKey)) == 0 {
			log.Info("Parition key was empty. Trying again to choose a non-empty partition key")
		} else {
			break
		}
	}
	id := dynamodb.AttributeValue{S: &partitionKey}

	for i := 0; i < 10; i++ {
		log.Infof("Attempt %d: Fetching existing item with partition key: %v", i, partitionKey)

		response, _ := ReadItem(client, tableName, id, metrics, true)
		if response.Item["id"] != nil {
			log.Infof("Successfully read existing item with partition key: %v", partitionKey)
			break
		}

		log.Errorf("Unable to find existing item with partition key: %v", partitionKey)
		if i == 9 {
			log.Errorf("All attempts to fetch the existing item with partition key: %v failed!", partitionKey)
			metrics.Successful = false
		}
	}
}

func SimulateWriteOperation(client *dax.Dax, tableName string, attributes int, metrics *models.DynamoDbSimulationMetrics) {
	log.Info("Performing WRITE operation...")
	benchmarkingItem, err := PutItem(client, tableName, attributes, metrics)
	if err != nil {
		log.Errorf("Unable to complete PUT simulation. %v+", err)
		metrics.Successful = false
		return
	}

	id := *benchmarkingItem["id"]

	AssertItemWasCreated(client, tableName, id, metrics)

	DeleteItem(client, tableName, id, metrics)

	AssertItemWasDeleted(client, tableName, id, metrics)
}

func SimulateUpdateOperation(client *dax.Dax, tableName string, attributes int, metrics *models.DynamoDbSimulationMetrics) {
	log.Info("Performing UPDATE operation...")
	newItem, err := PutItem(client, tableName, attributes, metrics)
	if err != nil {
		log.Errorf("Unable to complete UPDATE simulation. %v+", err)
		metrics.Successful = false
		return
	}

	id := *newItem["id"]
	partitionKey := *id.S
	attemptsExhausted := false

	AssertItemWasCreated(client, tableName, id, metrics)
	UpdateItem(client, tableName, id, attributes, metrics)

	startTime := time.Now()
	for i := 0; i < 10; i++ {
		log.Infof("Attempt %d: Fetching updated item for partition key: %v...", i, partitionKey)

		updatedItem, err := ReadItem(client, tableName, id, metrics, false)
		if err != nil {
			log.Errorf("Unable to complete UPDATE simulation. %v+", err)
			metrics.Successful = false
			return
		}

		if *newItem["1"].N != *updatedItem.Item["1"].N {
			log.Infof("Confirmed update for partition key: %v", partitionKey)
			break
		} else {
			log.Errorf("Update for partition key %v failed! Values are still equal!", partitionKey)
			if i == 9 {
				log.Error("Exhausted attempts to fetch updated item!")
				metrics.Successful = false
				attemptsExhausted = true
			}
		}
	}

	if !attemptsExhausted {
		duration := time.Since(startTime).Microseconds()
		millisecondDuration := float64(duration) / 1000
		metrics.UpdateItemConfirmationTime = &millisecondDuration
	}

	DeleteItem(client, tableName, id, metrics)
	AssertItemWasDeleted(client, tableName, id, metrics)
}
