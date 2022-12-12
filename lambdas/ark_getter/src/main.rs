use std::{collections::HashMap, env};

use aws_sdk_dynamodb::{error::QueryError, model::AttributeValue, output::QueryOutput, Client};
use aws_smithy_client::SdkError;
use lambda_http::{http::StatusCode, service_fn, Error, IntoResponse, Request, RequestExt};
use serde::Serialize;
use serde_json::json;

#[derive(Serialize)]
struct ScannerLog {
    created_at: u64,
    location: String,
    device_count: u32,
}

impl From<&HashMap<String, AttributeValue>> for ScannerLog {
    fn from(map: &HashMap<String, AttributeValue>) -> Self {
        Self {
            created_at: map
                .get("CreatedAt")
                .unwrap()
                .as_n()
                .unwrap()
                .to_owned()
                .parse()
                .unwrap(),
            location: map.get("Location").unwrap().as_s().unwrap().to_owned(),
            device_count: map
                .get("DeviceCount")
                .unwrap()
                .as_n()
                .unwrap()
                .to_owned()
                .parse()
                .unwrap(),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let config = aws_config::load_from_env().await;
    let client = Client::new(&config);
    let table_name = env::var("TABLE_NAME")?;

    lambda_http::run(service_fn(|request| handler(&client, &table_name, request))).await?;

    Ok(())
}

async fn handler(
    dynamo_client: &Client,
    table_name: &String,
    request: Request,
) -> Result<impl IntoResponse, Error> {
    let query_str = request.query_string_parameters();
    let start_param = query_str.first("start");
    let end_param = query_str.first("end");
    let location_param = query_str.first("location");

    let (start_str, end_str, location) = match (start_param, end_param, location_param) {
        (Some(start), Some(end), Some(location)) => (start, end, location),
        _ => return Ok((StatusCode::BAD_REQUEST, json!({}))),
    };

    let (start, end): (u64, u64) = match (start_str.parse(), end_str.parse()) {
        (Ok(s), Ok(e)) => (s, e),
        _ => {
            return Ok((
                StatusCode::BAD_REQUEST,
                json!({ "message": "start and end parameters should be non-negative integer" }),
            ))
        }
    };

    match get_item(dynamo_client, table_name, location, start, end).await {
        Ok(q) => {
            let items: Vec<ScannerLog> = q
                .items()
                .unwrap_or_default()
                .iter()
                .map(|v| v.into())
                .collect();

            return Ok((StatusCode::OK, json!({ "logs": items })));
        }
        Err(e) => Ok((
            StatusCode::INTERNAL_SERVER_ERROR,
            json!({ "message": e.to_string() }),
        )),
    }
}

async fn get_item<'a>(
    dynamo_client: &Client,
    table_name: &String,
    location: &'a str,
    start: u64,
    end: u64,
) -> Result<QueryOutput, SdkError<QueryError>> {
    dynamo_client
        .query()
        .table_name(table_name)
        .key_condition_expression("#loc = :pk AND #created_at BETWEEN :start AND :end")
        .expression_attribute_names("#loc", "Location")
        .expression_attribute_names("#created_at", "CreatedAt")
        .expression_attribute_values(":pk", AttributeValue::S(String::from(location)))
        .expression_attribute_values(":start", AttributeValue::N(start.to_string()))
        .expression_attribute_values(":end", AttributeValue::N(end.to_string()))
        .send()
        .await
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::handler;

    use testing::{mock_dynamo_client, unwrap_lambda_response, SdkBody};

    use lambda_http::{
        http::{Request, Response, StatusCode},
        Body, RequestExt,
    };
    use serde_json::Value;

    fn mock_request() -> Request<Body> {
        Request::builder()
            .body(Body::Text(String::new()))
            .unwrap()
            .with_query_string_parameters(HashMap::from_iter(vec![
                (String::from("location"), String::from("building-1")),
                (String::from("start"), String::from("0")),
                (String::from("end"), String::from("200")),
            ]))
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
            &String::from("Logs"),
            mock_request(),
        )
        .await;

        let (code, v): (StatusCode, Value) = unwrap_lambda_response(res.unwrap());

        eprintln!("{}", v);
        assert_eq!(code, StatusCode::OK);
    }

    #[tokio::test]
    #[should_panic]
    async fn handler_internal_fail() {
        let res = handler(
            &mock_dynamo_client(vec![(
                Request::builder().body(SdkBody::from("")).unwrap(),
                Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(SdkBody::from(""))
                    .unwrap(),
            )]),
            &String::from("Logs"),
            mock_request(),
        )
        .await;

        let (code, v): (StatusCode, Value) = unwrap_lambda_response(res.unwrap());

        eprintln!("{}", v);
        assert_eq!(code, StatusCode::OK);
    }
}
