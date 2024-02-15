use crate::data::store::Store;

use sam_e_types::config::{Config, EventProperties, EventType};

use fancy_regex::Regex;
use std::collections::HashMap;
use tracing::{debug, info};

#[derive(Debug, Clone)]
pub struct ApiState {
    pub sam_routes: Option<HashMap<String, Route>>,
    pub invocation_store: Store,
}

impl ApiState {
    pub fn new(config: &Config) -> Self {
        let sam_routes = create_routes(config);

        if let Some(sam_routes) = &sam_routes {
            info!("SAM routes detected, will pass into API state");
            debug!("SAM routes: {:#?}", sam_routes);
        }

        Self {
            invocation_store: Store::new(&sam_routes),
            sam_routes,
        }
        // if let Ok(sam_routes) = sam_routes {
        //     info!("SAM routes detected, will pass into API state");
        //     debug!("SAM routes: {:#?}", sam_routes);
        //     Self {
        //         sam_routes: Some(sam_routes),
        //         invocation_store: Store::new(&sam_routes),
        //     }
        // } else {
        //     info!("No SAM routes detected");
        //     Self {
        //         sam_routes: None,
        //         invocation_store: Store::new(),
        //     }
        // }
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

#[derive(Debug, Clone)]
pub struct Route {
    pub route: String,
    pub method: String,
    pub container_name: String,
    pub route_regex: Regex,
    pub route_base_path: Option<String>,
}

impl Route {
    pub fn create(
        route: String,
        method: String,
        container_name: String,
        route_regex: Regex,
        route_base_path: Option<String>,
    ) -> Self {
        Route {
            route,
            method,
            container_name,
            route_regex,
            route_base_path,
        }
    }
    pub fn get_route_base_path(&self) -> Option<&str> {
        self.route_base_path.as_deref()
    }
}

fn create_routes(config: &Config) -> Option<HashMap<String, Route>> {
    let mut routes = HashMap::new();
    // let api_lambdas: Vec<&Lambda> = config
    config.get_lambdas().iter().for_each(|lambda| {
        let lambda_name = lambda.get_name();
        let lambda_events = lambda.get_events();
        lambda_events.iter().for_each(|event| {
            if event.get_event_type() == &EventType::Api {
                let props = event.get_properties();

                if let Some(props) = props {
                    match props {
                        EventProperties::Api(api_props) => {
                            let base_path = api_props.get_base_path();
                            let path = api_props.get_path();
                            let method = api_props.get_method();

                            let replaced_path = replaced_regex_path(&path, base_path);
                            let route_regex = Regex::new(&replaced_path).expect("invalid regex");

                            let route = Route::create(
                                path.to_owned(),
                                method.to_owned(),
                                lambda_name.to_string(),
                                route_regex.to_owned(),
                                base_path.cloned(),
                            );

                            routes.insert(format!("{}::{}", path, method), route);
                        }
                        _ => {
                            debug!("Event detected as non-API event, skipping...");
                        }
                    }
                }
            }
        });
    });

    Some(routes)
}

fn replaced_regex_path(path: &str, base_path: Option<&String>) -> String {
    // As SAM supports parameters in url with {param} syntax we need to replace them with usable regex
    let replace_matches: Regex = Regex::new("{.*?}").expect("invalid regex");

    let replaced_sam_path =
        replace_matches
            .find_iter(path)
            .fold(path.to_string(), |mut acc, current_match| {
                if let Ok(current_match) = current_match {
                    let current_match_name: &str =
                        &current_match.as_str()[1..current_match.as_str().len() - 1];

                    if current_match_name.ends_with('+') {
                        acc = acc.replace(
                            current_match.as_str(),
                            &format!(
                                r"(?P<{}>.*)",
                                &current_match_name[0..current_match_name.len() - 1]
                            ),
                        );
                    } else {
                        acc = acc.replace(
                            current_match.as_str(),
                            &format!(r"(?P<{}>[^\/]+)", &current_match_name),
                        );
                    }
                };
                acc
            });

    if let Some(base_path) = base_path {
        format!("^/{}{}$", base_path, replaced_sam_path)
    } else {
        format!("^{}$", replaced_sam_path)
    }
}
