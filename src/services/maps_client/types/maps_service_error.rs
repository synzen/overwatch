pub enum MapsServiceError {
    Internal(String),
}

impl std::fmt::Display for MapsServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            MapsServiceError::Internal(e) => write!(f, "Internal error: {}", e),
        }
    }
}
