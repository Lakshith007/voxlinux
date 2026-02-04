#[derive(Debug)]
pub enum Opinion {
    Ok,
    Degraded { reason: String },
    Broken { reason: String },
}
