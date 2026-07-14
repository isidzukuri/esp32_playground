# DataAcquisitionEngine (DaqEngine)

Manages background threads for concurrent sensor data collection and lifecycle orchestration.

- Ingestion: Spawns threads to continuously read raw data from sensors.

- Processing: Aggregates and formats the incoming data streams.

- Dispatch: Hands off the processed data to the storage module for persistence.

Optimized for saving disk space by writing new data entries only if differ from the last stored.

## Tests
run tests on host linux machine:
```
cargo test --target x86_64-unknown-linux-gnu
```