use crate::openapi::spec::Spec;
use actix_web::{
    web::{self, get},
    HttpRequest, HttpResponse, Scope,
};

/// Application state for the Actix Web server.
pub struct AppState {
    pub spec: Spec,
}

/// Returns a new Actix Web scope with all the routes for the server.
pub fn get_scope() -> Scope {
    web::scope("").default_service(get().to(handle_all))
}

async fn handle_all(req: HttpRequest, data: web::Data<AppState>) -> HttpResponse {
    let spec = &data.spec;
    let example = spec.get_example(&req);

    match example {
        Some(example) => HttpResponse::Ok().json(example),
        None => HttpResponse::NotFound().finish(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, App};

    #[actix_rt::test]
    async fn test_request_default() {
        let spec = Spec::from_path("tests/testdata/petstore.yaml").expect("failed to load spec");
        let data = web::Data::new(AppState { spec });
        let app = App::new().app_data(data.clone()).service(get_scope());

        let mut app = test::init_service(app).await;
        let req = test::TestRequest::get().uri("/pets").to_request();
        let resp = test::call_service(&mut app, req).await;
        println!("{:?}", resp);
        assert!(resp.status().is_success());

        let expected_res = r#"[]"#;
        let body = test::read_body(resp).await;
        assert_eq!(body, expected_res);
    }

    #[actix_rt::test]
    async fn test_request_query() {
        let spec = Spec::from_path("tests/testdata/petstore.yaml").expect("failed to load spec");
        let data = web::Data::new(AppState { spec });
        let app = App::new().app_data(data.clone()).service(get_scope());

        let mut app = test::init_service(app).await;
        let req = test::TestRequest::get().uri("/pets?page=1").to_request();
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
        let spec = Spec::from_path("tests/testdata/petstore.yaml").expect("failed to load spec");
        let data = web::Data::new(AppState { spec });
        let app = App::new().app_data(data.clone()).service(get_scope());

        let mut app = test::init_service(app).await;
        let req = test::TestRequest::get().uri("/notfound").to_request();
        let resp = test::call_service(&mut app, req).await;
        assert!(resp.status().is_client_error());
    }
}
