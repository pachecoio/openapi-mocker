use actix_web::{
    web::{self, get},
    HttpRequest, HttpResponse, Scope,
};

use crate::spec::{load_endpoint, load_example, load_response, Method};

pub struct AppState {
    pub spec: oas3::OpenApiV3Spec,
}

pub fn get_scope() -> Scope {
    web::scope("").route("/{status}/{tail:.*}", get().to(handle_all))
}

async fn handle_all(req: HttpRequest, data: web::Data<AppState>) -> HttpResponse {
    let spec = &data.spec;
    let method = Method::from(req.method().as_str());
    let status = req
        .match_info()
        .get("status")
        .and_then(|v| v.parse().ok())
        .unwrap_or(200);

    let path = format!("/{}", req.match_info().get("tail").unwrap_or(""));

    let content_type = req
        .headers()
        .get("Content-Type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("application/json");

    match build_response(spec, &path, method, status, content_type) {
        Ok(resp) => resp,
        Err(e) => {
            eprintln!("Error: {:?}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

fn build_response(
    spec: &oas3::OpenApiV3Spec,
    path: &str,
    method: Method,
    status: u16,
    content_type: &str,
) -> Result<HttpResponse, Box<dyn std::error::Error>> {
    let op = load_endpoint(spec, path, method)?;
    let response = load_response(spec, &op, status)?;
    let result = load_example(spec, &response, content_type);

    let mut response_status = HttpResponse::build(get_status(status));
    Ok(response_status.json(result))
}

fn get_status(status: u16) -> actix_web::http::StatusCode {
    match status {
        200 => actix_web::http::StatusCode::OK,
        201 => actix_web::http::StatusCode::CREATED,
        202 => actix_web::http::StatusCode::ACCEPTED,
        204 => actix_web::http::StatusCode::NO_CONTENT,
        400 => actix_web::http::StatusCode::BAD_REQUEST,
        401 => actix_web::http::StatusCode::UNAUTHORIZED,
        403 => actix_web::http::StatusCode::FORBIDDEN,
        404 => actix_web::http::StatusCode::NOT_FOUND,
        405 => actix_web::http::StatusCode::METHOD_NOT_ALLOWED,
        406 => actix_web::http::StatusCode::NOT_ACCEPTABLE,
        409 => actix_web::http::StatusCode::CONFLICT,
        500 => actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
        _ => actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, App};

    #[actix_rt::test]
    async fn test_request_success() {
        let spec = oas3::from_path("tests/testdata/petstore.yaml").expect("failed to load spec");
        let data = web::Data::new(AppState { spec });
        let app = App::new().app_data(data.clone()).service(get_scope());

        let mut app = test::init_service(app).await;
        let req = test::TestRequest::get().uri("/200/pets").to_request();
        let resp = test::call_service(&mut app, req).await;
        println!("{:?}", resp);
        assert!(resp.status().is_success());

        let expected_res =
            r#"[{"id":1,"name":"doggie","tag":"dog"},{"id":2,"name":"kitty","tag":"cat"}]"#;
        let body = test::read_body(resp).await;
        assert_eq!(body, expected_res);
    }

    #[actix_rt::test]
    async fn test_request_not_found() {
        let spec = oas3::from_path("tests/testdata/petstore.yaml").expect("failed to load spec");
        let data = web::Data::new(AppState { spec });
        let app = App::new().app_data(data.clone()).service(get_scope());

        let mut app = test::init_service(app).await;
        let req = test::TestRequest::get().uri("/notfound").to_request();
        let resp = test::call_service(&mut app, req).await;
        assert!(resp.status().is_client_error());
    }
}
