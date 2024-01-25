use super::{sam::Route, store::Store};
use crate::data::sam::utils::create_sam_routes;
use std::collections::HashMap;
use tracing::{debug, info};

#[derive(Debug, Clone)]
pub struct ApiState {
    sam_routes: Option<HashMap<String, Route>>,
    invocation_store: Store,
}

impl ApiState {
    pub fn new(sam_template: &str) -> Self {
        let sam_routes = create_sam_routes(sam_template);

        if let Ok(sam_routes) = sam_routes {
            info!("SAM routes detected, will pass into API state");
            debug!("SAM routes: {:#?}", sam_routes);
            Self {
                sam_routes: Some(sam_routes),
                invocation_store: Store::new(),
            }
        } else {
            info!("No SAM routes detected");
            Self {
                sam_routes: None,
                invocation_store: Store::new(),
            }
        }
    }

    pub fn get_routes_vec(&self) -> Option<Vec<Route>> {
        if let Some(routes) = &self.sam_routes {
            Some(routes.values().cloned().collect())
        } else {
            None
        }
    }

    pub fn get_store(&self) -> &Store {
        &self.invocation_store
    }
}
