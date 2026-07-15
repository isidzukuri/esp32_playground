pub enum SensorType {
    Sound,
    Temperature,
    Humidity,
    Light
}

pub fn read_sensor(sensor_type: SensorType) -> f32 {
    match sensor_type {
        SensorType::Sound => rand::random_range(0.0..310.0),
        SensorType::Temperature => rand::random_range(5.0..20.0),
        SensorType::Humidity => rand::random_range(10.0..90.0),
        SensorType::Light => rand::random_range(0.0..1500.0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_sensor() {
        let result = read_sensor(SensorType::Sound);
        assert!(0.0 <= result);
        assert!(result <= 310.0);

        let result = read_sensor(SensorType::Temperature);
        assert!(5.0 <= result);
        assert!(result <= 20.0);

        let result = read_sensor(SensorType::Humidity);
        assert!(10.0 <= result);
        assert!(result <= 90.0);

        let result = read_sensor(SensorType::Light);
        assert!(0.0 <= result);
        assert!(result <= 1500.0);
    }
}
