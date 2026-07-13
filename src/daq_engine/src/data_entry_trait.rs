use std::collections::HashMap;

pub trait DataEntryTrait {
    fn ts(&self) -> &u64;
    fn increment_ts(&mut self);
    fn attrs(&mut self) -> &mut HashMap<String, f32>;
    fn data_to_log(&self, attrs_order: Option<Vec<String>>) -> String;
}

#[macro_export]
macro_rules! impl_daq_data_entry_trait {
    ($struct_type:ty) => {
        impl DataEntryTrait for $struct_type {
            fn ts(&self) -> &u64 {
                &self.ts
            }

            fn increment_ts(&mut self) {
                self.ts += 1;
            }

            fn attrs(&mut self) -> &mut HashMap<String, f32> {
                &mut self.attrs
            }

            fn data_to_log(&self, attrs_order: Option<Vec<String>>) -> String {
                match attrs_order {
                    Some(order) => {
                        let mut result = self.ts.to_string();
                        for attr in order {
                            result = format!("{},{}", result, self.attrs.get(&attr).unwrap_or(&0.0))
                        }
                        result
                    },
                    None => {
                        let attrs_values: Vec<f32> = self.attrs.values().copied().collect();
                        let val_string = attrs_values.iter().map(|val| val.to_string()).collect::<Vec<_>>().join(",");
                        format!("{},{}", self.ts, val_string)
                    }
                }
            }
        }
    };
}


#[cfg(test)]
mod tests {
    use super::*;

    const TIMESTAMP: u64 = 1767268800;

    #[derive(Default, Debug)]
    pub struct TestDataEntry {
        ts: u64,
        attrs: HashMap<String, f32>,
    }
    impl_daq_data_entry_trait!(TestDataEntry);

    #[test]
    fn test_initialize(){
        let mut inst = TestDataEntry {
            ts: TIMESTAMP,
            attrs: HashMap::from([("test".to_string(), 1.2)])
        };

        assert_eq!(*inst.ts(), TIMESTAMP);
        assert_eq!(inst.attrs().get("test").unwrap(), &1.2);
    }

    #[test]
    fn test_increment_ts(){
        let mut inst = TestDataEntry {
            ts: TIMESTAMP,
            attrs: HashMap::new()
        };

        assert_eq!(*inst.ts(), TIMESTAMP);
        inst.increment_ts();
        assert_eq!(*inst.ts(), TIMESTAMP+1);
    }

    #[test]
    fn test_data_to_log(){
        let mut inst = TestDataEntry {
            ts: TIMESTAMP,
            attrs: HashMap::new()
        };

        assert_eq!(inst.data_to_log(None), format!("{},", TIMESTAMP));
    }

    #[test]
    fn test_data_to_log_when_attrs_and_order(){
        let mut inst = TestDataEntry {
            ts: TIMESTAMP,
            attrs: HashMap::from([("test".to_string(), 1.1), ("test2".to_string(), 2.2), ("test3".to_string(), 3.3)])
        };

        let val_string = inst.data_to_log(Some(vec!["test2".to_string(), "test3".to_string(), "test4".to_string(), "test".to_string()]));
        assert_eq!(val_string, "1767268800,2.2,3.3,0,1.1".to_string());
    }
}
