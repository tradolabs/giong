use std::time::Duration;

pub(crate) const REQUEST_TIMEOUT: Duration = Duration::from_secs(120);
pub(crate) const DEFAULT_DOCKER_SOCKET: &str = "/var/run/docker.sock";
pub(crate) const DOCKER_API_URL: &str = "http://localhost";
