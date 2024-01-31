use std::time;

use gdriver_common::ipc::gdriver_service::{BackendActionRequest, GDriverServiceClient};

use super::*;

pub async fn start() -> Result<()> {
    println!("Hello, world!");
    let config = &CONFIGURATION;
    println!("Config: {:?}", **config);
    let client: GDriverServiceClient = create_client(config.ip, config.port).await?;

    let hello = client
        .do_something2(tarpc::context::current(), BackendActionRequest::Ping)
        .await;
    match hello {
        Ok(hello) => println!("Yay: {:?}", hello),
        Err(e) => {
            println!(":( {:?}", (e));
            dbg!(e);
        }
    }
    let start = time::SystemTime::now();
    let hello = client
        .do_something2(tarpc::context::current(), BackendActionRequest::RunLong)
        .await;

    let seconds = (time::SystemTime::now().duration_since(start))
        .unwrap()
        .as_secs();

    match hello {
        Ok(hello) => println!("Run Long returned after {} seconds: {:?}", seconds, hello),
        Err(e) => println!(":( {:?}", (e)),
    }
    let start = time::SystemTime::now();
    let hello = client
        .do_something2(tarpc::context::current(), BackendActionRequest::StartLong)
        .await;
    let seconds = (time::SystemTime::now().duration_since(start))
        .unwrap()
        .as_secs();

    match hello {
        Ok(hello) => println!("Start Long returned after {} seconds: {:?}", seconds, hello),
        Err(e) => println!(":( {:?}", (e)),
    }
    Ok(())
}
pub async fn create_client(ip: IpAddr, port: u16) -> Result<GDriverServiceClient> {
    let server_addr = (ip, port);
    let transport = tarpc::serde_transport::tcp::connect(&server_addr, Json::default)
        .await
        .map_err(|e| {
            println!("Could not connect");
            e
        })?;
    let service = GDriverServiceClient::new(client::Config::default(), transport);
    let client = service.spawn();
    Ok(client)
}
