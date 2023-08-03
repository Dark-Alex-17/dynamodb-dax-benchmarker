#!make

default: build

.PHONY: init start-elastic-stack stop-elastic-stack build build-dynamodb-benchmarker build-dax-benchmarker run-dynamodb-benchmarker run-dax-benchmarker clean lint

init: build
	@[[ -d ../docker-elk ]] || git clone https://github.com/deviantony/docker-elk.git ..
	@cd ../docker-elk && docker compose up setup
	@echo "Default login creds: username=elastic, password=changeme"

start-elastic-stack:
	@cd ../docker-elk && docker compose up -d

stop-elastic-stack:
	@cd ../docker-elk && docker compose down

build-dynamodb-benchmarker:
	@cargo clean && rm -f dynamodb-benchmarker && cargo build --release && mv ./target/release/dynamodb-benchmarker .

build-dax-benchmarker:
	@rm -f main && rm -f dax-benchmarker && go build -o dax-benchmarker pkg/app/main.go

build: build-dynamodb-benchmarker build-dax-benchmarker
	
run-dynamodb-benchmarker:
	@cargo run

run-dax-benchmarker:
	@go run pkg/app/main.go

clean:
	@cargo clean && rm -f main && rm -f dynamodb-benchmarker && rm -f dax-benchmarker && rm -rf cdk/cdk.out && rm -rf cdk/node_modules

lint:
	@cargo clippy && golangci-lint run