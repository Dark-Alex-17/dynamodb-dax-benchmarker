package models

import (
	"fmt"
	"math/rand"
	"strconv"
	"time"

	"github.com/aws/aws-sdk-go/service/dynamodb"
	"github.com/google/uuid"
	"gopkg.in/loremipsum.v1"
)

type DynamoOperation int

const (
	DynamoRead DynamoOperation = iota
	DynamoWrite
	DynamoUpdate
)

func (d DynamoOperation) String() string {
	switch d {
	case DynamoRead:
		return "read"
	case DynamoWrite:
		return "write"
	case DynamoUpdate:
		return "update"
	default:
		return "read"
	}
}

type Scenario int

const (
	ScenarioCrud Scenario = iota
	ScenarioReadOnly
)

func (s Scenario) String() string {
	switch s {
	case ScenarioCrud:
		return "crud"
	case ScenarioReadOnly:
		return "readOnly"
	default:
		return "crud"
	}
}

type BenchmarkingItem map[string]*dynamodb.AttributeValue

func NewBenchmarkingItem(attributes int) BenchmarkingItem {
	benchmarkingItem := make(map[string]*dynamodb.AttributeValue)
	r := rand.New(rand.NewSource(time.Now().UnixNano()))
	loremIpsumGenerator := loremipsum.NewWithSeed(time.Now().UnixNano())
	id := uuid.New().String()
	benchmarkingItem["id"] = &dynamodb.AttributeValue{S: &id}

	for i := 0; i < attributes; i++ {
		switch i % 2 {
		case 1:
			float := fmt.Sprintf("%.2f", r.Float64()*32.00)
			benchmarkingItem[strconv.Itoa(i)] = &dynamodb.AttributeValue{N: &float}
		default:
			sentence := loremIpsumGenerator.Sentence()
			benchmarkingItem[strconv.Itoa(i)] = &dynamodb.AttributeValue{S: &sentence}
		}
	}

	return benchmarkingItem
}

type DynamoDbSimulationMetrics struct {
	Operation                  string   `json:"operation"`
	Timestamp                  int64    `json:"timestamp"`
	Successful                 bool     `json:"successful"`
	Scenario                   string   `json:"scenario"`
	SimulationTime             *float64 `json:"simulationTime,omitempty"`
	ReadTime                   *float64 `json:"readTime,omitempty"`
	WriteTime                  *float64 `json:"writeTime,omitempty"`
	WriteItemConfirmationTime  *float64 `json:"writeItemConfirmationTime,omitempty"`
	UpdateTime                 *float64 `json:"updateItem,omitempty"`
	UpdateItemConfirmationTime *float64 `json:"updateItemConfirmationTime,omitempty"`
	DeleteTime                 *float64 `json:"deleteTime,omitempty"`
	DeleteItemConfirmationTime *float64 `json:"deleteItemConfirmationTime,omitempty"`
}
