use fancy_regex::Regex;

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
}
