use std::collections::HashMap;

pub trait DataEntryTrait {
    fn ts(&self) -> &u64;
    fn increment_ts(&mut self);
    fn attrs(&mut self) -> &mut HashMap<String, f32>;
    fn for_storage_channel(&self) -> (u64, HashMap<String, f32>);
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

            fn for_storage_channel(&self) -> (u64, HashMap<String, f32>) {
                (self.ts, self.attrs.clone())
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
    fn test_for_storage_channel(){
        let mut inst = TestDataEntry {
            ts: TIMESTAMP,
            attrs: HashMap::from([("test".to_string(), 1.2)])
        };

        assert_eq!(inst.for_storage_channel(), (TIMESTAMP, HashMap::from([("test".to_string(), 1.2)])));
    }
}
