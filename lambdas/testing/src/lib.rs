use std::any::Any;

use aws_sdk_dynamodb::{Client, Config, Credentials, Region};
use aws_smithy_client::{erase::DynConnector, test_connection::TestConnection};
pub use aws_smithy_http::body::SdkBody;
use lambda_http::{
    http::{Request, Response},
    IntoResponse,
};

type ConnectionsVec = Vec<(Request<SdkBody>, Response<SdkBody>)>;

pub fn mock_dynamo_config() -> Config {
    Config::builder()
        .region(Region::new("us-east-1"))
        .credentials_provider(Credentials::new(
            "access_key",
            "secret_key",
            None,
            None,
            "provider",
        ))
        .build()
}

pub fn mock_test_connection(connections: ConnectionsVec) -> TestConnection<SdkBody> {
    TestConnection::new(connections)
}

pub fn mock_dynamo_connector(connections: ConnectionsVec) -> DynConnector {
    DynConnector::new(mock_test_connection(connections))
}

pub fn mock_dynamo_client(connections: ConnectionsVec) -> Client {
    Client::from_conf_conn(mock_dynamo_config(), mock_dynamo_connector(connections))
}

/// Force unwrap ``impl Response`` by downcasting to concrete type
pub fn unwrap_lambda_response<T, U>(response: T) -> U
where
    T: IntoResponse + 'static,
    U: Clone + 'static,
{
    let any: Box<dyn Any> = Box::new(response);

    match any.downcast_ref::<U>() {
        Some(u) => u.clone(),
        None => panic!("Unreachable panic: unable to downcast"),
    }
}
