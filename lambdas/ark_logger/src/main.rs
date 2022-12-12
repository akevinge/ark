use aws_sdk_dynamodb::{error::PutItemError, model::AttributeValue, output::PutItemOutput, Client};
use aws_smithy_client::SdkError;
use lambda_http::{http::StatusCode, service_fn, Body, Error, IntoResponse, Request};
use serde::Deserialize;
use serde_json::{json, Value};

#[derive(Deserialize)]
struct ScannerLog {
    created_at: u64,
    location: String,
    device_count: u32,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let config = aws_config::load_from_env().await;
    let client = Client::new(&config);

    lambda_http::run(service_fn(|request| handler(&client, request))).await?;

    Ok(())
}

async fn handler(dynamo_client: &Client, request: Request) -> Result<impl IntoResponse, Error> {
    let body = match request.body() {
        Body::Text(text) => text,
        _ => return malformed_body(),
    };

    let scanner_log: ScannerLog = match serde_json::from_str(&body) {
        Ok(l) => l,
        Err(_) => return malformed_body(),
    };

    match put_item(dynamo_client, scanner_log).await {
        Ok(_) => Ok((StatusCode::CREATED, json!({}))),
        Err(_) => Ok((
            StatusCode::INTERNAL_SERVER_ERROR,
            json!({ "message": "unable to create log" }),
        )),
    }
}

fn malformed_body() -> Result<(StatusCode, Value), Error> {
    Ok((
        StatusCode::BAD_REQUEST,
        json!({ "message": "malformed body" }),
    ))
}

async fn put_item(
    dynamo_client: &Client,
    log: ScannerLog,
) -> Result<PutItemOutput, SdkError<PutItemError>> {
    dynamo_client
        .put_item()
        .table_name("Logs")
        .item("Location", AttributeValue::S(log.location))
        .item("CreatedAt", AttributeValue::N(log.created_at.to_string()))
        .item(
            "DeviceCount",
            AttributeValue::N(log.device_count.to_string()),
        )
        .send()
        .await
}

#[cfg(test)]
mod tests {
    use super::handler;

    use testing::{mock_dynamo_client, unwrap_lambda_response, SdkBody};

    use lambda_http::{
        http::{Method, Request, Response, StatusCode},
        Body,
    };
    use serde_json::{json, Value};

    fn mock_request() -> Request<Body> {
        Request::builder()
            .method(Method::POST)
            .body(Body::Text(
                json!({ "created_at": 1, "location": "building-1", "device_count": 200 })
                    .to_string(),
            ))
            .unwrap()
    }

    #[tokio::test]
    async fn handler_ok() {
        let res = handler(
            &mock_dynamo_client(vec![(
                Request::builder().body(SdkBody::from("")).unwrap(),
                Response::builder()
                    .status(StatusCode::OK)
                    .body(SdkBody::from(""))
                    .unwrap(),
            )]),
            mock_request(),
        )
        .await;

        let (code, v): (StatusCode, Value) = unwrap_lambda_response(res.unwrap());

        eprintln!("{}", v);
        assert_eq!(code, StatusCode::CREATED);
    }

    #[tokio::test]
    #[should_panic]
    async fn handler_fail() {
        let res = handler(
            &mock_dynamo_client(vec![(
                Request::builder().body(SdkBody::from("")).unwrap(),
                Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(SdkBody::from(""))
                    .unwrap(),
            )]),
            mock_request(),
        )
        .await;

        let (code, v): (StatusCode, Value) = unwrap_lambda_response(res.unwrap());

        eprintln!("{}", v);
        assert_eq!(code, StatusCode::CREATED);
    }
}
