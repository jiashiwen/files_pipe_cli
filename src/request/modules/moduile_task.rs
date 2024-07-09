use super::{
    module_filter::LastModifyFilter, module_storage::ObjectStorage,
    module_task_compare::CompareTask,
};
use crate::commons::{byte_size_str_to_usize, byte_size_usize_to_str};
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use snowflake::SnowflakeIdGenerator;

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum TransferType {
    Full,
    Stock,
    Increment,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
// #[serde(untagged)]
#[serde(rename_all = "lowercase")]
#[serde(tag = "type")]
pub enum Task {
    Transfer(TransferTask),
    Compare(CompareTask),
    // TruncateBucket(TaskTruncateBucket),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub struct TransferTask {
    #[serde(default = "TaskDefaultParameters::id_default")]
    pub task_id: String,
    #[serde(default = "TaskDefaultParameters::name_default")]
    pub name: String,
    pub source: ObjectStorage,
    pub target: ObjectStorage,
    pub attributes: TransferTaskAttributes,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TransferTaskAttributes {
    #[serde(default = "TaskDefaultParameters::objects_per_batch_default")]
    pub objects_per_batch: i32,
    #[serde(default = "TaskDefaultParameters::task_parallelism_default")]
    pub task_parallelism: usize,
    #[serde(default = "TaskDefaultParameters::max_errors_default")]
    pub max_errors: usize,
    #[serde(default = "TaskDefaultParameters::meta_dir_default")]
    pub meta_dir: String,
    #[serde(default = "TaskDefaultParameters::target_exists_skip_default")]
    pub target_exists_skip: bool,
    #[serde(default = "TaskDefaultParameters::start_from_checkpoint_default")]
    pub start_from_checkpoint: bool,
    #[serde(default = "TaskDefaultParameters::large_file_size_default")]
    #[serde(serialize_with = "se_usize_to_str")]
    #[serde(deserialize_with = "de_usize_from_str")]
    pub large_file_size: usize,
    #[serde(default = "TaskDefaultParameters::multi_part_chunk_size_default")]
    #[serde(serialize_with = "se_usize_to_str")]
    #[serde(deserialize_with = "de_usize_from_str")]
    pub multi_part_chunk_size: usize,
    #[serde(default = "TaskDefaultParameters::multi_part_chunks_per_batch_default")]
    pub multi_part_chunks_per_batch: usize,
    #[serde(default = "TaskDefaultParameters::multi_part_parallelism_default")]
    pub multi_part_parallelism: usize,
    #[serde(default = "TaskDefaultParameters::multi_part_parallelism_default")]
    pub multi_part_max_parallelism: usize,
    #[serde(default = "TaskDefaultParameters::filter_default")]
    pub exclude: Option<Vec<String>>,
    #[serde(default = "TaskDefaultParameters::filter_default")]
    pub include: Option<Vec<String>>,
    #[serde(default = "TaskDefaultParameters::transfer_type_default")]
    pub transfer_type: TransferType,
    #[serde(default = "TaskDefaultParameters::last_modify_filter_default")]
    pub last_modify_filter: Option<LastModifyFilter>,
}

pub fn de_usize_from_str<'de, D>(deserializer: D) -> Result<usize, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    byte_size_str_to_usize(&s).map_err(de::Error::custom)
}

pub fn se_usize_to_str<S>(v: &usize, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let size = byte_size_usize_to_str(*v);
    serializer.serialize_str(size.as_str())
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CompareTaskAttributes {
    #[serde(default = "TaskDefaultParameters::objects_per_batch_default")]
    pub objects_per_batch: i32,
    #[serde(default = "TaskDefaultParameters::task_parallelism_default")]
    pub task_parallelism: usize,
    #[serde(default = "TaskDefaultParameters::max_errors_default")]
    pub max_errors: usize,
    #[serde(default = "TaskDefaultParameters::meta_dir_default")]
    pub meta_dir: String,
    #[serde(default = "TaskDefaultParameters::target_exists_skip_default")]
    pub start_from_checkpoint: bool,
    #[serde(default = "TaskDefaultParameters::large_file_size_default")]
    #[serde(serialize_with = "se_usize_to_str")]
    #[serde(deserialize_with = "de_usize_from_str")]
    pub large_file_size: usize,
    #[serde(default = "TaskDefaultParameters::multi_part_chunk_size_default")]
    #[serde(serialize_with = "se_usize_to_str")]
    #[serde(deserialize_with = "de_usize_from_str")]
    pub multi_part_chunk: usize,
    #[serde(default = "TaskDefaultParameters::filter_default")]
    pub exclude: Option<Vec<String>>,
    #[serde(default = "TaskDefaultParameters::filter_default")]
    pub include: Option<Vec<String>>,
    #[serde(default = "TaskDefaultParameters::exprirs_diff_scope_default")]
    pub exprirs_diff_scope: i64,
    #[serde(default = "TaskDefaultParameters::continuous_default")]
    pub continuous: bool,
    #[serde(default = "TaskDefaultParameters::last_modify_filter_default")]
    pub last_modify_filter: Option<LastModifyFilter>,
}
pub struct TaskDefaultParameters {}

impl TaskDefaultParameters {
    pub fn id_default() -> String {
        task_id_generator().to_string()
    }

    pub fn name_default() -> String {
        "default_name".to_string()
    }

    pub fn objects_per_batch_default() -> i32 {
        100
    }

    pub fn task_parallelism_default() -> usize {
        num_cpus::get()
    }

    pub fn max_errors_default() -> usize {
        1
    }

    pub fn start_from_checkpoint_default() -> bool {
        false
    }

    pub fn exprirs_diff_scope_default() -> i64 {
        10
    }

    pub fn target_exists_skip_default() -> bool {
        false
    }
    pub fn large_file_size_default() -> usize {
        // 50M
        10485760 * 5
    }
    pub fn multi_part_chunk_size_default() -> usize {
        // 10M
        10485760
    }

    pub fn multi_part_chunks_per_batch_default() -> usize {
        10
    }
    pub fn multi_part_parallelism_default() -> usize {
        num_cpus::get() + 2
    }

    pub fn multi_part_max_parallelism_default() -> usize {
        num_cpus::get() * 2
    }

    pub fn meta_dir_default() -> String {
        "/tmp/meta_dir".to_string()
    }

    pub fn filter_default() -> Option<Vec<String>> {
        None
    }
    pub fn continuous_default() -> bool {
        false
    }

    pub fn transfer_type_default() -> TransferType {
        TransferType::Stock
    }

    pub fn last_modify_filter_default() -> Option<LastModifyFilter> {
        None
    }
}

pub fn task_id_generator() -> i64 {
    let mut id_generator_generator = SnowflakeIdGenerator::new(1, 1);
    let id = id_generator_generator.real_time_generate();
    id
}
