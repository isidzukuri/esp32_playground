#[derive(Debug, Clone)]
pub struct SensorReader {
    pub name: String,
    pub wait_ms: u64,
    pub toleration_percentage: f32,
    pub function: fn() -> f32
}