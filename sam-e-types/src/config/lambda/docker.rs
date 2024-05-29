use serde::{Deserialize, Serialize};
use tracing::{debug, trace};

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct DockerBuild {
    pub dockerfile: String,
    pub context: String,
    pub use_ssh: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DockerBuildBuilder {
    dockerfile: String,
    context: String,
    use_ssh: Option<bool>,
}

impl DockerBuildBuilder {
    pub fn new() -> Self {
        Self {
            dockerfile: String::new(),
            context: String::new(),
            use_ssh: None,
        }
    }

    pub fn with_dockerfile(mut self, dockerfile: String) -> Self {
        trace!("Setting dockerfile to: {}", dockerfile);
        self.dockerfile = dockerfile;
        self
    }

    pub fn with_context(mut self, context: String) -> Self {
        trace!("Setting context to: {}", context);
        self.context = context;
        self
    }

    pub fn with_use_ssh(mut self, use_ssh: bool) -> Self {
        trace!("Setting use_ssh to: {}", use_ssh);
        self.use_ssh = Some(use_ssh);
        self
    }

    pub fn build(self) -> DockerBuild {
        debug!("Building docker build settings");
        let using_ssh = if let Some(use_ssh) = self.use_ssh {
            use_ssh
        } else {
            debug!("use_ssh not set, defaulting to false");
            false
        };

        DockerBuild {
            dockerfile: self.dockerfile,
            context: self.context,
            use_ssh: using_ssh,
        }
    }
}
