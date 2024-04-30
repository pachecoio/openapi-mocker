use oas3::spec::{Operation, PathItem, Response};

pub type SpecResult<T> = Result<T, Box<dyn std::error::Error>>;

/// HTTP methods
pub enum Method {
    Get,
    Post,
    Put,
    Delete,
    Options,
    Head,
    Patch,
    Trace,
}

impl From<&str> for Method {
    fn from(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "GET" => Method::Get,
            "POST" => Method::Post,
            "PUT" => Method::Put,
            "DELETE" => Method::Delete,
            "OPTIONS" => Method::Options,
            "HEAD" => Method::Head,
            "PATCH" => Method::Patch,
            "TRACE" => Method::Trace,
            _ => panic!("Invalid method"),
        }
    }
}

/// Load an OpenAPI spec from a file
///
/// # Arguments
/// * `path` - Path to the OpenAPI spec file
///
/// # Returns
/// An OpenAPI spec object
///
/// # Example
/// ```
/// use openapi_mocker::spec::load_spec;
///
/// let spec = load_spec("tests/testdata/petstore.yaml");
/// assert_eq!(spec.openapi, "3.0.0");
/// ```
pub fn load_spec(path: &str) -> oas3::OpenApiV3Spec {
    oas3::from_path(path).unwrap()
}

/// Load an endpoint from an OpenAPI spec
///
/// # Arguments
/// * `spec` - OpenAPI spec object
/// * `path` - Path to the endpoint
/// * `method` - HTTP method
///
/// # Returns
/// An OpenAPI operation object
///
/// # Example
/// ```
/// use openapi_mocker::spec::{load_spec, load_endpoint, Method};
///
/// let spec = load_spec("tests/testdata/petstore.yaml");
/// let op = load_endpoint(&spec, "/pets", Method::Get).unwrap();
/// assert_eq!(op.operation_id, Some("listPets".to_string()));
/// ```
pub fn load_endpoint(
    spec: &oas3::OpenApiV3Spec,
    path: &str,
    method: Method,
) -> SpecResult<Operation> {
    let op = spec
        .paths
        .get(path)
        .and_then(load_method(method))
        .ok_or("Endpoint not found")?;
    Ok(op.clone())
}

/// Load a method from a PathItem
///
/// # Arguments
/// * `method` - HTTP method
/// * `path_item` - PathItem object
///
/// # Returns
/// An Option with the Operation object
fn load_method<'a>(method: Method) -> impl Fn(&PathItem) -> Option<&Operation> + 'a {
    move |path_item: &PathItem| match method {
        Method::Get => path_item.get.as_ref(),
        Method::Post => path_item.post.as_ref(),
        Method::Put => path_item.put.as_ref(),
        Method::Delete => path_item.delete.as_ref(),
        Method::Options => path_item.options.as_ref(),
        Method::Head => path_item.head.as_ref(),
        Method::Patch => path_item.patch.as_ref(),
        Method::Trace => path_item.trace.as_ref(),
    }
}

/// Load a response from an OpenAPI operation
///
/// # Arguments
/// * `spec` - OpenAPI spec object
/// * `op` - OpenAPI operation object
/// * `status` - HTTP status code
///
/// # Returns
/// An OpenAPI response object
///
/// # Example
/// ```
/// use openapi_mocker::spec::{load_spec, load_endpoint, load_response, Method};
///
/// let spec = load_spec("tests/testdata/petstore.yaml");
/// let op = load_endpoint(&spec, "/pets", Method::Get).unwrap();
/// let response = load_response(&spec, &op, 200).unwrap();
/// assert_eq!(response.description, Some("A paged array of pets".to_string()));
/// ```
pub fn load_response(
    spec: &oas3::OpenApiV3Spec,
    op: &Operation,
    status: u16,
) -> Result<oas3::spec::Response, Box<dyn std::error::Error>> {
    let status_str = status.to_string();
    let objorref = op.responses.get(&status_str).ok_or("Response not found")?;

    match objorref.resolve(&spec) {
        Ok(r) => Ok(r),
        Err(_) => Err("Response not found".into()),
    }
}

/// Load an example from an OpenAPI response
///
/// # Arguments
/// * `spec` - OpenAPI spec object
/// * `response` - OpenAPI response object
/// * `content_type` - Content type
///
/// # Returns
/// A JSON value with the example
///
/// # Example
/// ```
/// use openapi_mocker::spec::{load_spec, load_endpoint, load_response, load_example, Method};
/// use serde_json::json;
///
/// let spec = load_spec("tests/testdata/petstore.yaml");
/// let op = load_endpoint(&spec, "/pets", Method::Get).unwrap();
/// let response = load_response(&spec, &op, 200).unwrap();
/// let content_type = "application/json";
/// let example = load_example(&spec, &response, content_type).unwrap();
/// let expected = json!([
///     {
///         "id": 1,
///         "name": "doggie",
///         "tag": "dog"
///         },
///     {
///         "id": 2,
///         "name": "kitty",
///         "tag": "cat"
///     }
/// ]);
/// assert_eq!(example, expected);
/// ```
pub fn load_example(
    spec: &oas3::OpenApiV3Spec,
    response: &Response,
    content_type: &str,
) -> Option<serde_json::Value> {
    response
        .content
        .get(content_type)
        .expect("Content not found")
        .schema
        .as_ref()
        .expect("Schema not found")
        .resolve(&spec)
        .expect("Failed to resolve schema")
        .example
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_spec() {
        let spec = load_spec("tests/testdata/petstore.yaml");
        assert_eq!(spec.openapi, "3.0.0");
    }

    #[test]
    fn test_load_endpoint() {
        let spec = load_spec("tests/testdata/petstore.yaml");
        let op = load_endpoint(&spec, "/pets", Method::Get).unwrap();
        assert_eq!(op.operation_id, Some("listPets".to_string()));
    }

    #[test]
    fn test_load_endpoint_not_found() {
        let spec = load_spec("tests/testdata/petstore.yaml");
        let op = load_endpoint(&spec, "/notfound", Method::Get);
        assert!(op.is_err());
    }

    #[test]
    fn test_load_response() {
        let spec = load_spec("tests/testdata/petstore.yaml");
        let op = load_endpoint(&spec, "/pets", Method::Get).unwrap();
        let response = load_response(&spec, &op, 200).unwrap();
        assert_eq!(
            response.description,
            Some("A paged array of pets".to_string())
        );
    }

    #[test]
    fn test_load_response_not_found() {
        let spec = load_spec("tests/testdata/petstore.yaml");
        let op = load_endpoint(&spec, "/pets", Method::Get).unwrap();
        let response = load_response(&spec, &op, 404);
        assert!(response.is_err());
    }

    #[test]
    fn test_load_example() {
        let spec = load_spec("tests/testdata/petstore.yaml");
        let op = load_endpoint(&spec, "/pets", Method::Get).unwrap();

        let response = load_response(&spec, &op, 200).unwrap();
        let content_type = "application/json";
        let example = load_example(&spec, &response, content_type).unwrap();
        let example_json = serde_json::to_string(&example).unwrap();

        let expected = serde_json::json!([
            {
                "id": 1,
                "name": "doggie",
                "tag": "dog"
            },
            {
                "id": 2,
                "name": "kitty",
                "tag": "cat"
            }
        ]);
        let expected_json = serde_json::to_string(&expected).unwrap();

        assert_eq!(example_json, expected_json);
    }

    #[test]
    fn test_load_example_string() {
        let spec = load_spec("tests/testdata/petstore.yaml");
        let op = load_endpoint(&spec, "/pets", Method::Get).unwrap();

        let response = load_response(&spec, &op, 200).unwrap();
        let content_type = "text/plain";
        let example = load_example(&spec, &response, content_type).unwrap();
        let example_json = serde_json::to_string(&example).unwrap();

        let expected = serde_json::json!("Not implemented");
        let expected_json = serde_json::to_string(&expected).unwrap();

        assert_eq!(example_json, expected_json);
    }
}
