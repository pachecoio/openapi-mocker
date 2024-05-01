use actix_web::HttpRequest;
use oas3::spec::{MediaTypeExamples, ObjectOrReference, Operation, PathItem, Response};

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
        let example_name = "default";

        Some(&self.spec)
            .and_then(load_path(path))
            .and_then(load_method(&method))
            .and_then(load_responses())
            .and_then(load_examples(&self.spec, media_type))
            .and_then(|examples| examples.into_iter().next())
            .and_then(get_example(example_name, &self.spec))
    }
}

fn load_spec(path: &str) -> Option<oas3::OpenApiV3Spec> {
    match oas3::from_path(path) {
        Ok(spec) => Some(spec),
        Err(_) => None,
    }
}

fn load_path<'a>(path: &'a str) -> impl Fn(&oas3::OpenApiV3Spec) -> Option<PathItem> + 'a {
    move |spec: &oas3::OpenApiV3Spec| spec.paths.clone().get(path).cloned()
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
    use actix_web::test::TestRequest;

    use super::*;

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
    fn test_load_responses() {
        let responses = load_spec("tests/testdata/petstore.yaml")
            .as_ref()
            .and_then(load_path("/pets"))
            .and_then(load_method("get"))
            .and_then(load_responses());
        assert!(responses.is_some());
        assert_eq!(responses.unwrap().len(), 2);
    }
}
