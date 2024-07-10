use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub enum LastModifyFilterType {
    Greater,
    Less,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct LastModifyFilter {
    pub filter_type: LastModifyFilterType,
    pub timestamp: i64,
}

// impl LastModifyFilter {
//     pub fn filter(&self, timestamp: i128) -> bool {
//         match self.filter_type {
//             LastModifyFilterType::Greater => timestamp.ge(&self.timestamp),
//             LastModifyFilterType::Less => timestamp.le(&self.timestamp),
//         }
//     }
// }
