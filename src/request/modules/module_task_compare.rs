use serde::{Deserialize, Serialize};

use crate::request::modules::module_task::TaskDefaultParameters;

use super::module_storage::ObjectStorage;
use super::module_task::CompareTaskAttributes;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub struct CompareTask {
    #[serde(default = "TaskDefaultParameters::id_default")]
    pub task_id: String,
    #[serde(default = "TaskDefaultParameters::name_default")]
    pub name: String,
    pub source: ObjectStorage,
    pub target: ObjectStorage,
    pub check_option: CompareCheckOption,
    pub attributes: CompareTaskAttributes,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CompareCheckOption {
    #[serde(default = "CompareCheckOption::default_check_content_length")]
    check_content_length: bool,
    #[serde(default = "CompareCheckOption::default_check_expires")]
    check_expires: bool,
    #[serde(default = "CompareCheckOption::default_check_content")]
    check_content: bool,
    #[serde(default = "CompareCheckOption::default_check_meta_data")]
    check_meta_data: bool,
}

impl Default for CompareCheckOption {
    fn default() -> Self {
        Self {
            check_content_length: CompareCheckOption::default_check_content_length(),
            check_expires: CompareCheckOption::default_check_expires(),
            check_content: CompareCheckOption::default_check_content(),
            check_meta_data: CompareCheckOption::default_check_meta_data(),
        }
    }
}

impl CompareCheckOption {
    pub fn default_check_content_length() -> bool {
        true
    }

    pub fn default_check_expires() -> bool {
        false
    }

    pub fn default_check_content() -> bool {
        false
    }

    pub fn default_check_meta_data() -> bool {
        false
    }

    pub fn check_content_length(&self) -> bool {
        self.check_content_length
    }

    pub fn check_expires(&self) -> bool {
        self.check_expires
    }

    pub fn check_meta_data(&self) -> bool {
        self.check_meta_data
    }

    pub fn check_content(&self) -> bool {
        self.check_content
    }
}
