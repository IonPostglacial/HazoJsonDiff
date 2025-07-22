#[cfg_attr(test, derive(Debug))]
pub enum JsonDiffErrorType {
    InvalidStructureObjectKey,
    PropertyMissing,
    InvalidStructureUnclosed,
    InvalidStructureUnexpectedToken,
    InvalidStructureInvalidNumber,
}

#[cfg_attr(test, derive(Debug))]
pub struct JsonDiffError {
    #[allow(dead_code)]
    pub error_type: JsonDiffErrorType,
}