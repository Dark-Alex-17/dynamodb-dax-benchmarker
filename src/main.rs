use std::{env, time::Duration};

use anyhow::anyhow;

use aws_sdk_dynamodb::Client;
use chrono::Utc;
use clap::Parser;
use elasticsearch::{
  auth::Credentials,
  http::{
    transport::{SingleNodeConnectionPool, TransportBuilder},
    Url,
  },
  indices::IndicesPutMappingParts,
  Elasticsearch,
};
use log::{error, info, warn, LevelFilter};
use log4rs::{
  append::console::ConsoleAppender,
  config::{Appender, Root},
  encode::pattern::PatternEncoder,
};
use models::{DynamoDbSimulationMetrics, DynamoOperation};
use rand::{
  rngs::{OsRng, StdRng},
  Rng, SeedableRng,
};
use serde_json::json;
use tokio::{
  select,
  sync::mpsc::{self, Receiver, Sender},
  task::JoinHandle,
};
use tokio_util::sync::CancellationToken;

use crate::{models::Scenario, simulators::Simulator};

mod models;
mod simulators;
mod timer_utils;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
  /// The number of concurrent simulations to run
  #[arg(short, long, default_value_t = 1000)]
  concurrent_simulations: u32,
  /// The number of attributes to use when populating and querying the DynamoDB table; minimum value of 1
  #[arg(short, long, default_value_t = 5)]
  attributes: u32,
  /// The length of time (in seconds) to run the benchmark for
  #[arg(short, long, default_value_t = 1800)]
  duration: u64,
  /// The buffer size of the Elasticsearch thread's MPSC channel
  #[arg(short, long, default_value_t = 500)]
  buffer: usize,
  /// Local Elasticsearch cluster username
  #[arg(short, long, default_value_t = String::from("elastic"))]
  username: String,
  /// Local Elasticsearch cluster password
  #[arg(short, long, default_value_t = String::from("changeme"))]
  password: String,
  /// The Elasticsearch Index to insert data into
  #[arg(short, long, default_value_t = String::from("dynamodb"))]
  index: String,
  /// The DynamoDB table to perform operations against
  #[arg(short, long, default_value_t = format!("{}-high-velocity-table", env::var("USER").unwrap()))]
  table_name: String,
  /// Whether to run a read-only scenario for benchmarking
  #[arg(short, long)]
  read_only: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  let cli = Cli::parse();
  log4rs::init_config(init_logging_config())?;
  let cancellation_token = CancellationToken::new();

  let (es_tx, es_rx) = mpsc::channel::<DynamoDbSimulationMetrics>(cli.buffer);
  std::thread::spawn(move || {
    start_elasticsearch_publisher(es_rx, cli.username, cli.password, cli.index)
  });

  let handles: Vec<JoinHandle<_>> = (0..cli.concurrent_simulations)
    .map(|_| {
      let tx = es_tx.clone();
      let token = cancellation_token.clone();
      let table_name = cli.table_name.clone();

      tokio::spawn(async move {
        let config = aws_config::load_from_env().await;
        let dynamodb_client = Client::new(&config);
        match scan_all_partition_keys(&dynamodb_client, table_name.clone()).await {
          Ok(partition_keys_vec) => {
            let simulator = Simulator::new(
              &dynamodb_client,
              table_name.clone(),
              cli.attributes,
              &partition_keys_vec,
            );
            select! {
              _ = token.cancelled() => {
                warn!("Task cancelled. Shutting down...");
              }
              _ = simulation_loop(simulator, cli.read_only, tx) => ()
            }
          }
          Err(e) => error!("Unable to fetch partition keys: {e:?}"),
        }
      })
    })
    .collect();

  tokio::spawn(async move {
    info!(
      "Starting timer task. Executing for {} seconds",
      cli.duration
    );

    tokio::time::sleep(Duration::from_secs(cli.duration)).await;

    cancellation_token.cancel();
  });

  for handle in handles {
    match handle.await {
      Ok(_) => info!("Task shut down gracefully"),
      Err(e) => warn!("Task did not shut down gracefully {e:?}"),
    }
  }

  Ok(())
}

#[tokio::main]
async fn start_elasticsearch_publisher(
  mut elasticsearch_rx: Receiver<DynamoDbSimulationMetrics>,
  username: String,
  password: String,
  index: String,
) -> anyhow::Result<()> {
  let url = Url::parse("http://localhost:9200")?;
  let connection_pool = SingleNodeConnectionPool::new(url);
  let credentials = Credentials::Basic(username, password);
  let transport = TransportBuilder::new(connection_pool)
    .auth(credentials)
    .build()?;
  let es_client = Elasticsearch::new(transport);

  info!("Setting the explicit mappings for the {index} index");
  es_client
    .indices()
    .put_mapping(IndicesPutMappingParts::Index(&[&index]))
    .body(json!({
      "properties": {
        "timestamp": {
          "type": "date"
        }
      }
    }))
    .send()
    .await?;

  while let Some(metric) = elasticsearch_rx.recv().await {
    info!("Publishing metrics to Elasticsearch...");

    let es_response = es_client
      .index(elasticsearch::IndexParts::Index(&index))
      .body(metric)
      .send()
      .await;

    match es_response {
      Ok(resp) => {
        if resp.status_code().is_success() {
          info!("Successfully published metrics to Elasticsearch");
        } else {
          error!("Was unable to publish metrics to Elasticsearch! Received non 2XX response");
        }
      }
      Err(e) => {
        error!("Unable to publish metrics to Elasticsearch! {e:?}");
      }
    }
  }

  Ok(())
}

async fn simulation_loop(
  mut simulator: Simulator<'_>,
  read_only: bool,
  tx: Sender<DynamoDbSimulationMetrics>,
) {
  let mut rng = StdRng::from_seed(OsRng.gen());
  loop {
    let mut metrics = DynamoDbSimulationMetrics::default();
    metrics.timestamp = Utc::now();

    let simulation_time = time!(match {
      if read_only {
        info!("Running a read-only simulation...");
        metrics.scenario = Scenario::ReadOnly;
        run_read_only_simulation(&mut simulator, &mut metrics, &mut rng).await
      } else {
        info!("Running a CRUD simulation...");
        metrics.scenario = Scenario::Crud;
        run_crud_simulation(&mut simulator, &mut metrics, &mut rng).await
      }
    } {
      Ok(_) => {
        info!("Simulation completed successfully!");
        metrics.successful = true;
      }
      Err(e) => error!("Simulation did not complete. Encountered the following error: {e:?}"),
    });
    metrics.simulation_time = Some(simulation_time);
    info!("Metrics: {metrics:?}");

    match tx.send(metrics).await {
      Ok(_) => info!("Metrics sent down channel successfully"),
      Err(e) => error!("Metrics were unable to be sent down the channel! {e:?}"),
    }
  }
}

async fn run_read_only_simulation(
  simulator: &mut Simulator<'_>,
  metrics: &mut DynamoDbSimulationMetrics,
  rng: &mut StdRng,
) -> anyhow::Result<()> {
  tokio::time::sleep(Duration::from_secs(rng.gen_range(0..15))).await;

  metrics.operation = DynamoOperation::Read;
  simulator.simulate_read_operation(metrics).await?;

  Ok(())
}

async fn run_crud_simulation(
  simulator: &mut Simulator<'_>,
  metrics: &mut DynamoDbSimulationMetrics,
  rng: &mut StdRng,
) -> anyhow::Result<()> {
  match DynamoOperation::from(rng.gen_range(0..3)) {
    DynamoOperation::Read => {
      metrics.operation = DynamoOperation::Read;
      simulator.simulate_read_operation(metrics).await?
    }
    DynamoOperation::Write => {
      metrics.operation = DynamoOperation::Write;
      simulator.simulate_write_operation(metrics).await?;
    }
    DynamoOperation::Update => {
      metrics.operation = DynamoOperation::Update;
      simulator.simulate_update_operation(metrics).await?;
    }
  }

  Ok(())
}

async fn scan_all_partition_keys(
  dynamodb_client: &Client,
  table_name: String,
) -> anyhow::Result<Vec<String>> {
  info!("Fetching a large list of partition keys to randomly read...");
  let response = dynamodb_client
    .scan()
    .table_name(table_name)
    .limit(10000)
    .projection_expression("id")
    .send()
    .await;

  match response {
    Ok(resp) => {
      info!("Fetched partition keys!");
      let partition_keys = resp
        .items()
        .unwrap()
        .into_iter()
        .map(|attribute| {
          attribute
            .values()
            .last()
            .unwrap()
            .as_s()
            .unwrap()
            .to_string()
        })
        .collect::<Vec<String>>();
      info!("Found a total of {} keys", partition_keys.len());
      Ok(partition_keys)
    }
    Err(e) => {
      error!("Unable to fetch partition keys! {e:?}");
      Err(anyhow!(e))
    }
  }
}

fn init_logging_config() -> log4rs::Config {
  let stdout = ConsoleAppender::builder()
    .encoder(Box::new(PatternEncoder::new(
      "{d(%Y-%m-%d %H:%M:%S%.3f)(utc)} <{i}> [{l}] {f}:{L} - {m}{n}",
    )))
    .build();

  log4rs::Config::builder()
    .appender(Appender::builder().build("stdout", Box::new(stdout)))
    .build(Root::builder().appender("stdout").build(LevelFilter::Info))
    .unwrap()
}
