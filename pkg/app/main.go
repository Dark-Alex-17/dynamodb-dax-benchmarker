package main

import (
	"bytes"
	"encoding/json"
	"errors"
	"fmt"
	"math/rand"
	"os"
	"strconv"
	"strings"
	"time"

	"github.com/Dark-Alex-17/dynamodb-benchmarker/pkg/models"
	"github.com/Dark-Alex-17/dynamodb-benchmarker/pkg/simulators"
	"github.com/Dark-Alex-17/dynamodb-benchmarker/pkg/utils"
	"github.com/aws/aws-dax-go/dax"
	"github.com/aws/aws-sdk-go/aws"
	"github.com/aws/aws-sdk-go/aws/credentials"
	"github.com/aws/aws-sdk-go/aws/session"
	"github.com/aws/aws-sdk-go/service/dynamodb"
	"github.com/elastic/go-elasticsearch/v8"
	log "github.com/sirupsen/logrus"
	"github.com/spf13/cobra"
)

var concurrentSimulations, buffer, attributes, duration int
var username, password, index, endpoint, tableName string
var readOnly bool

func main() {
	rootCmd := &cobra.Command{
		Use:   "dax-benchmarker",
		Short: "A CLI tool for simulating heavy usage against DAX and publishing metrics to an Elastic Stack for analysis",
		RunE: func(cmd *cobra.Command, args []string) error {
			if err := validateFlags(); err != nil {
				return err
			}

			execute()
			return nil
		},
	}
	rootCmd.PersistentFlags().IntVarP(&concurrentSimulations, "concurrent-simulations", "c", 1000, "The number of concurrent simulations to run")
	rootCmd.PersistentFlags().IntVarP(&buffer, "buffer", "b", 500, "The buffer size of the Elasticsearch goroutine's channel")
	rootCmd.PersistentFlags().IntVarP(&attributes, "attributes", "a", 5, "The number of attributes to use when populating and querying the DynamoDB table; minimum value of 1")
	rootCmd.PersistentFlags().IntVarP(&duration, "duration", "d", 1800, "The length of time (in seconds) to run the benchmark for")
	rootCmd.PersistentFlags().StringVarP(&username, "username", "u", "elastic", "Local Elasticsearch cluster username")
	rootCmd.PersistentFlags().StringVarP(&password, "password", "p", "changeme", "Local Elasticsearch cluster password")
	rootCmd.PersistentFlags().StringVarP(&index, "index", "i", "dax", "The Elasticsearch Index to insert data into")
	rootCmd.PersistentFlags().StringVarP(&tableName, "table", "t", fmt.Sprintf("%s-high-velocity-table", os.Getenv("USER")), "The DynamoDB table to perform operations against")
	rootCmd.PersistentFlags().StringVarP(&endpoint, "endpoint", "e", "", "The DAX endpoint to hit when running simulations (assumes secure endpoint, so do not specify port)")
	rootCmd.PersistentFlags().BoolVarP(&readOnly, "read-only", "r", false, "Whether to run a read-only scenario for benchmarking")

	if err := rootCmd.Execute(); err != nil {
		log.Errorf("Something went wrong parsing CLI args and executing the client: %v", err)
	}
}

func validateFlags() error {
	if len(endpoint) == 0 {
		daxEndpointEnvironmentVariable := os.Getenv("DAX_ENDPOINT")
		if len(daxEndpointEnvironmentVariable) == 0 {
			return errors.New("a DAX endpoint must be specified either via -e, --endpoint or via the DAX_ENDPOINT environment variable")
		} else {
			endpoint = daxEndpointEnvironmentVariable
		}
	}

	if attributes < 1 {
		return errors.New("the number of attributes cannot be lower than 1")
	}

	if len(os.Getenv("AWS_REGION")) == 0 {
		return errors.New("an AWS region must be specified using the AWS_REGION environment variable")
	}

	return nil
}

func execute() {
	esChan := make(chan models.DynamoDbSimulationMetrics, buffer)
	defer close(esChan)
	daxEndpoint := fmt.Sprintf("%s:9111", endpoint)
	region := os.Getenv("AWS_REGION")
	sess := session.Must(session.NewSession(&aws.Config{
		Credentials: credentials.NewChainCredentials([]credentials.Provider{&credentials.EnvProvider{}}),
		Endpoint:    &daxEndpoint,
		Region:      &region,
	}))

	if _, err := sess.Config.Credentials.Get(); err != nil {
		log.Errorf("credentials were not loaded! %v+", err)
	}

	client, err := dax.NewWithSession(*sess)
	if err != nil {
		log.Errorf("unable to initialize dax client %v", err)
	}

	partitionKeys, err := scanAllPartitionKeys(client)
	if err != nil {
		log.Errorf("Unable to fetch partition keys! Simulation failed! %v+", err)
	}

	go startElasticsearchPublisher(esChan)

	for i := 0; i < concurrentSimulations; i++ {
		go simulationLoop(esChan, client, partitionKeys)
	}

	duration, err := time.ParseDuration(strconv.Itoa(duration) + "s")
	if err != nil {
		log.Errorf("Unable to create duration from the provided time: %v", err)
		return
	}

	<-time.After(duration)
}

func startElasticsearchPublisher(c <-chan models.DynamoDbSimulationMetrics) {
	config := elasticsearch.Config{
		Addresses: []string{
			"http://localhost:9200",
		},
		Username: username,
		Password: password,
	}
	esClient, err := elasticsearch.NewClient(config)
	if err != nil {
		log.Errorf("unable to initialize elasticsearch client %v", err)
	}

	mapping := `{
		"properties": {
			"timestamp": {
				"type": "date"
			}
		}
	}`

	log.Infof("Setting the explicit mappings for the %s index", index)
	if _, err := esClient.Indices.Create(index); err != nil {
		log.Warnf("Unable to create the %s index. Encountered the following error: %v", index, err)
	}

	if _, err := esClient.Indices.PutMapping([]string{index}, strings.NewReader(mapping)); err != nil {
		log.Errorf("unable to create mapping for the %s index! %v+", index, err)
	}

	for metric := range c {
		log.Info("Publishing metrics to Elasticsearch...")

		data, _ := json.Marshal(metric)
		_, err := esClient.Index(index, bytes.NewReader(data))
		if err != nil {
			log.Error("Was unable to publish metrics to Elasticsearch! Received a non 2XX response")
		} else {
			log.Info("Successfully published metrics to Elasticsearch")
		}
	}
}

func simulationLoop(c chan<- models.DynamoDbSimulationMetrics, client *dax.Dax, partitionKeys []string) {
	for {
		metrics := new(models.DynamoDbSimulationMetrics)
		metrics.Successful = true
		metrics.Timestamp = time.Now().UnixNano() / 1e6
		startTime := time.Now()

		if readOnly {
			log.Info("Running a read-only simulation...")
			metrics.Scenario = models.ScenarioReadOnly.String()
			runReadOnlySimulation(client, metrics, partitionKeys)
		} else {
			log.Info("Running a CRUD simulation...")
			metrics.Scenario = models.ScenarioCrud.String()
			runCrudSimulation(client, metrics, partitionKeys)
		}

		log.Info("Simulation completed successfully!")

		duration := time.Since(startTime).Microseconds()
		millisecondDuration := float64(duration) / 1000
		metrics.SimulationTime = &millisecondDuration

		log.Infof("Metrics: %v+", metrics)

		c <- *metrics
	}
}

func runReadOnlySimulation(client *dax.Dax, metrics *models.DynamoDbSimulationMetrics, partitionKeys []string) {
	r := rand.New(rand.NewSource(time.Now().UnixNano()))
	time.Sleep(time.Duration(r.Intn(16)))

	metrics.Operation = models.DynamoRead.String()
	simulators.SimulateReadOperation(client, tableName, partitionKeys, metrics)
}

func runCrudSimulation(client *dax.Dax, metrics *models.DynamoDbSimulationMetrics, partitionKeys []string) {
	r := rand.New(rand.NewSource(time.Now().UnixNano()))
	operation := r.Intn(3)
	log.Infof("Operation number: %d", operation)

	switch operation {
	case int(models.DynamoRead):
		metrics.Operation = models.DynamoRead.String()
		simulators.SimulateReadOperation(client, tableName, partitionKeys, metrics)
	case int(models.DynamoWrite):
		metrics.Operation = models.DynamoWrite.String()
		simulators.SimulateWriteOperation(client, tableName, attributes, metrics)
	case int(models.DynamoUpdate):
		metrics.Operation = models.DynamoUpdate.String()
		simulators.SimulateUpdateOperation(client, tableName, attributes, metrics)
	}
}

func scanAllPartitionKeys(client *dax.Dax) ([]string, error) {
	log.Info("Fetching a large list of partition keys to randomly read...")
	projectionExpression := "id"
	var limit int64 = 10000

	response, err := client.Scan(&dynamodb.ScanInput{
		TableName:            &tableName,
		Limit:                &limit,
		ProjectionExpression: &projectionExpression,
	})

	if err != nil {
		log.Errorf("Unable to fetch partition keys! %v", err)
		return []string{}, err
	} else {
		log.Info("Fetched partition keys!")
		keys := make([]string, 100)

		for _, itemsMap := range response.Items {
			keys = append(keys, *utils.MapValues(itemsMap)[0].S)
		}

		log.Infof("Found a total of %d keys", len(keys))

		return keys, nil
	}
}
