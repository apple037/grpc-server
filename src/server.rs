use tonic::{transport::Server, Request, Response, Status};
use grpcserver::{TestRequest, TestResponse, grpc_test_service_server::{GrpcTestService, GrpcTestServiceServer}};
use chrono::Local;
use prost_types::Timestamp;

pub mod grpcserver {
    tonic::include_proto!("com.jasper.test.grpc.grpcserver");
}

#[derive(Debug, Default)]
pub struct MyGrpcTestService {}

fn i64_timestamp_to_proto_timestamp(timestamp: i64) -> Timestamp {
    let seconds = timestamp / 1000;
    let nanos = (timestamp % 1000) * 1_000_000;
    Timestamp {
        seconds,
        nanos: nanos as i32,
        ..Default::default() 
    }
}

fn proto_timestamp_to_i64_timestamp(timestamp: Timestamp) -> i64 {
    let seconds = timestamp.seconds;
    let nanos = timestamp.nanos as i64;
    (seconds * 1000) + (nanos / 1_000_000)
}

#[tonic::async_trait]
impl GrpcTestService for MyGrpcTestService {
    async fn test_connection_time_cost(&self, request: Request<TestRequest>) -> Result<Response<TestResponse>, Status> {
        println!("Got a request: {:?}", request);
        let request = request.into_inner();
        let send_time = request.send_time_stamp.expect("send_time_stamp is None");
        let send_time_in_i64 = proto_timestamp_to_i64_timestamp(send_time.clone());
        println!("send_time_stamp: {:?}", send_time);
        println!("send_time_in_i64: {:?}", send_time_in_i64);
        // Get current time
        let now = Local::now();
        let now_timestamp_in_millisconds = now.timestamp() * 1000 + now.timestamp_subsec_millis() as i64;
        println!("now_timestamp: {:?}", now_timestamp_in_millisconds);
        let now_timestamp_in_proto = i64_timestamp_to_proto_timestamp(now_timestamp_in_millisconds);
        println!("now_timestamp_in_proto: {:?}", now_timestamp_in_proto);
        
        // Calculate the time cost
        let time_cost = now_timestamp_in_millisconds - send_time_in_i64;
        println!("time_cost: {:?}", time_cost);
        let time_cost_in_proto = i64_timestamp_to_proto_timestamp(time_cost);
        println!("time_cost_in_proto: {:?}", time_cost_in_proto.clone());

        let response = TestResponse {
            send_time_stamp: Some(send_time),
            receive_time_stamp: Some(now_timestamp_in_proto),
            process_time_cost: Some(time_cost_in_proto.clone()),
        };
        Ok(Response::new(response))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "127.0.0.1:9998".parse()?;
    let grpc_test_service = MyGrpcTestService::default();

    println!("Grpc server is running on: {:?}", addr);
    Server::builder()
        .add_service(GrpcTestServiceServer::new(grpc_test_service))
        .serve(addr)
        .await?;

    Ok(())
}

