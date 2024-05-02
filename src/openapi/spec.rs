use std::collections::HashMap;

use actix_web::HttpRequest;
use oas3::spec::{Example, MediaTypeExamples, ObjectOrReference, Operation, PathItem, Response};

pub type SpecResult<T> = Result<T, Box<dyn std::error::Error>>;

pub struct Spec {
    spec: oas3::OpenApiV3Spec,
}

impl Spec {
    pub fn from_path(path: &str) -> SpecResult<Self> {
        let spec = load_spec(path).ok_or("Failed to load spec")?;
        Ok(Self { spec })
    }

    pub fn get_example(&self, req: &HttpRequest) -> Option<serde_json::Value> {
        let path = req.uri().path();
        let method = req.method().as_str().to_lowercase();
        let media_type = "application/json";

        Some(&self.spec)
            .and_then(load_path(path))
            .and_then(load_method(&method))
            .and_then(load_responses())
            .and_then(load_examples(&self.spec, media_type))
            .and_then(find_example_match(req))
            .and_then(|example| example.resolve(&self.spec).ok())
            .and_then(|example| example.value)
    }
}

fn load_spec(path: &str) -> Option<oas3::OpenApiV3Spec> {
    match oas3::from_path(path) {
        Ok(spec) => Some(spec),
        Err(_) => None,
    }
}

fn load_path<'a>(path: &'a str) -> impl Fn(&oas3::OpenApiV3Spec) -> Option<PathItem> + 'a {
    move |spec: &oas3::OpenApiV3Spec| {
        spec.paths
            .iter()
            .find(|(key, _)| match_url(path, &[*key]))
            .map(|(_, value)| value.clone())
    }
}

fn match_url(url: &str, routes: &[&str]) -> bool {
    let url_parts: Vec<&str> = url.split('/').filter(|s| !s.is_empty()).collect();

    for route in routes {
        let route_parts: Vec<&str> = route.split('/').filter(|s| !s.is_empty()).collect();
        if url_parts.len() == route_parts.len()
            && route_parts
                .iter()
                .zip(url_parts.iter())
                .all(|(r, u)| r.starts_with('{') && r.ends_with('}') || r == u)
        {
            return true;
        }
    }
    false
}

fn load_method<'a>(method: &'a str) -> impl Fn(PathItem) -> Option<Operation> + 'a {
    move |path: PathItem| match method {
        "get" => path.get.clone(),
        "put" => path.put.clone(),
        "post" => path.post.clone(),
        "delete" => path.delete.clone(),
        "options" => path.options.clone(),
        "head" => path.head.clone(),
        "patch" => path.patch.clone(),
        "trace" => path.trace.clone(),
        _ => None,
    }
}

fn load_responses<'a>() -> impl Fn(Operation) -> Option<Vec<ObjectOrReference<Response>>> + 'a {
    move |op: Operation| {
        let mut responses = Vec::new();
        for (_, response) in op.responses.iter() {
            responses.push(response.clone());
        }
        Some(responses)
    }
}

fn load_examples<'a>(
    spec: &'a oas3::OpenApiV3Spec,
    media_type: &'a str,
) -> impl Fn(Vec<ObjectOrReference<Response>>) -> Option<Vec<MediaTypeExamples>> + 'a {
    move |responses: Vec<ObjectOrReference<Response>>| {
        let mut examples = Vec::new();
        for response in responses {
            extract_response(response, spec)
                .as_ref()
                .and_then(|r| r.content.get(media_type))
                .and_then(|content| content.examples.as_ref())
                .map(|media_type| examples.push(media_type.clone()));
        }
        Some(examples)
    }
}

fn extract_response(
    response: ObjectOrReference<Response>,
    spec: &oas3::OpenApiV3Spec,
) -> Option<Response> {
    match response {
        ObjectOrReference::Object(response) => Some(response),
        ObjectOrReference::Ref { ref_path } => {
            let components = &spec.components;
            components
                .as_ref()
                .and_then(|components| components.responses.get(&ref_path).cloned())
                .and_then(|resp| extract_response(resp, spec))
        }
    }
}

/// Find the example that matches the request.
///
/// It matches the examples by comparing the request path, query,
/// and headers with the example name.
/// If the example name matches the request path, it returns the example.
/// If the example name does not match the request path, it returns None.
///
/// # Matching exact route
/// If the example name is the same as the request path, it returns the example.
/// Example:
/// - Example name: `/pets`
/// - Request path: `/pets`
/// - Returns the example
///
/// - Example name: `/pets`
/// - Request path: `/pets/123`
/// - Returns None
fn find_example_match<'a>(
    req: &'a HttpRequest,
) -> impl Fn(Vec<MediaTypeExamples>) -> Option<ObjectOrReference<Example>> {
    let path = req.uri().path().to_string();

    let query = QueryString::from_request(req);

    move |examples: Vec<MediaTypeExamples>| {
        let mut default: Option<ObjectOrReference<Example>> = None;
        for example in examples {
            match example {
                MediaTypeExamples::Examples { examples } => {
                    for (example_name, e) in examples.iter() {
                        // Match exact path
                        if example_name == &path {
                            return Some(e.clone());
                        }

                        // Match query parameters
                        if query.match_example(&example_name) {
                            return Some(e.clone());
                        }

                        // Match default example
                        if example_name == "default" {
                            default = Some(e.clone());
                        }
                    }
                }
                _ => {}
            }
        }
        default
    }
}

struct QueryString {
    params: HashMap<String, String>,
}

impl QueryString {
    fn from_request(req: &HttpRequest) -> Self {
        let mut params = HashMap::new();
        for (key, value) in req.query_string().split('&').map(|pair| {
            let mut split = pair.split('=');
            (split.next().unwrap(), split.next().unwrap_or(""))
        }) {
            params.insert(key.to_string(), value.to_string());
        }
        Self { params }
    }

    fn match_example(&self, example_name: &str) -> bool {
        if example_name.starts_with("query:") {
            let query = example_name.trim_start_matches("query:");
            let mut query_params = HashMap::new();
            for pair in query.split('&').map(|pair| {
                let mut split = pair.split('=');
                (split.next().unwrap(), split.next().unwrap_or(""))
            }) {
                query_params.insert(pair.0.to_string(), pair.1.to_string());
            }
            self.params
                .iter()
                .all(|(key, value)| query_params.get(key).map_or(false, |v| v == value))
        } else {
            false
        }
    }
}

fn get_example<'a>(
    example_name: &'a str,
    spec: &'a oas3::OpenApiV3Spec,
) -> impl Fn(MediaTypeExamples) -> Option<serde_json::Value> + 'a {
    move |examples: MediaTypeExamples| match examples {
        MediaTypeExamples::Examples { examples } => examples
            .get(example_name)
            .map(|example| example.resolve(spec))
            .map(|example| match example {
                Ok(example) => example.value,
                Err(_) => None,
            })
            .flatten(),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::test::TestRequest;

    #[test]
    fn test_load_spec() {
        let spec = load_spec("tests/testdata/petstore.yaml");
        assert_eq!(spec.unwrap().openapi, "3.0.0");
    }

    #[test]
    fn test_load_path() {
        let path = load_spec("tests/testdata/petstore.yaml")
            .as_ref()
            .and_then(load_path("/pets"));
        assert!(path.is_some());
    }

    #[test]
    fn test_load_path_not_found() {
        let path = load_spec("tests/testdata/petstore.yaml")
            .as_ref()
            .and_then(load_path("/notfound"));
        assert!(path.is_none());
    }

    #[test]
    fn test_load_path_with_params() {
        let path = load_spec("tests/testdata/petstore.yaml")
            .as_ref()
            .and_then(load_path("/pets/{petId}"));
        assert!(path.is_some());
    }

    #[test]
    fn test_load_path_with_dynamic_params() {
        let path = load_spec("tests/testdata/petstore.yaml")
            .as_ref()
            .and_then(load_path("/pets/123"));
        assert!(path.is_some());
    }

    #[test]
    fn test_load_method() {
        let method = load_spec("tests/testdata/petstore.yaml")
            .as_ref()
            .and_then(load_path("/pets"))
            .and_then(load_method("get"));
        assert!(method.is_some());
    }

    #[test]
    fn test_load_method_not_found() {
        let method = load_spec("tests/testdata/petstore.yaml")
            .as_ref()
            .and_then(load_path("/pets"))
            .and_then(load_method("notfound"));
        assert!(method.is_none());
    }

    #[test]
    fn test_load_examples() {
        let spec = load_spec("tests/testdata/petstore.yaml").unwrap();

        let example = Some(&spec)
            .and_then(load_path("/pets"))
            .and_then(load_method("get"))
            .and_then(load_responses())
            .and_then(load_examples(&spec, "application/json"));
        assert!(example.is_some());
    }

    #[test]
    fn test_spec() {
        let spec = Spec::from_path("tests/testdata/petstore.yaml").unwrap();
        let req = TestRequest::with_uri("/pets").to_http_request();
        let example = spec.get_example(&req);
        assert!(example.is_some());
    }

    #[test]
    fn test_spec_with_path_params() {
        let spec = Spec::from_path("tests/testdata/petstore.yaml").unwrap();
        let req = TestRequest::with_uri("/pets/123").to_http_request();
        let example = spec.get_example(&req);
        assert!(example.is_some());
    }

    #[test]
    fn test_spec_with_params_custom_example() {
        let spec = Spec::from_path("tests/testdata/petstore.yaml").unwrap();
        let req = TestRequest::with_uri("/pets/2").to_http_request();
        let example = spec.get_example(&req).unwrap();

        assert_eq!(
            example["id"],
            serde_json::Value::Number(serde_json::Number::from(2))
        );
    }

    #[test]
    fn test_spec_match_query_params() {
        let spec = Spec::from_path("tests/testdata/petstore.yaml").unwrap();
        let req = TestRequest::with_uri("/pets?page=1").to_http_request();
        let res = spec.get_example(&req).unwrap();

        let example = res.as_array().unwrap().get(0).unwrap();
        assert_eq!(
            example["id"],
            serde_json::Value::Number(serde_json::Number::from(1))
        );
    }

    #[test]
    fn test_spec_match_query_params_with_multiple_params() {
        let spec = Spec::from_path("tests/testdata/petstore.yaml").unwrap();
        let req = TestRequest::with_uri("/pets?page=1&limit=1").to_http_request();
        let res = spec.get_example(&req).unwrap();

        let examples = res.as_array().unwrap();
        assert_eq!(examples.len(), 1,);
        let example = examples.get(0).unwrap();
        assert_eq!(
            example["id"],
            serde_json::Value::Number(serde_json::Number::from(1))
        );
    }

    #[test]
    fn test_spec_prefer_path_over_query_params() {
        let spec = Spec::from_path("tests/testdata/petstore.yaml").unwrap();
        let req = TestRequest::with_uri("/pets/2?term=dog").to_http_request();
        let example = spec.get_example(&req).unwrap();
        assert_eq!(
            example["id"],
            serde_json::Value::Number(serde_json::Number::from(2))
        );
    }
}
