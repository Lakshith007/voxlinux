#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SystemState {
    PackageConsistent,
    ServiceActive(String),

    // NEW STATES
    NetworkReachable,
    FilesystemWritable(String),
}

