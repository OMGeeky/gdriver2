use std::time;

use gdriver_common::ipc::gdriver_service::{BackendActionRequest, GDriverServiceClient};

use super::*;

pub async fn start() -> Result<()> {
    println!("Hello, world!");
    let config = &CONFIGURATION;
    println!("Config: {:?}", **config);
    let client: GDriverServiceClient = create_client(config.ip, config.port).await?;
    run_long_stuff_test(&client).await;
    Ok(())
}

async fn ping(client: &GDriverServiceClient) -> Result<()> {
    let hello = client
        .do_something2(tarpc::context::current(), BackendActionRequest::Ping)
        .await;
    match hello {
        Ok(hello) => info!("Yay: {:?}", hello),
        Err(e) => {
            error!(":( {:?}", (e));
            dbg!(&e);
            return Err(Box::new(e));
        }
    }
    Ok(())
}
#[allow(unused)]
async fn run_long_stuff_test(client: &GDriverServiceClient) {
    let start = time::SystemTime::now();
    let hello = client
        .do_something2(tarpc::context::current(), BackendActionRequest::RunLong)
        .await;

    let seconds = (time::SystemTime::now().duration_since(start))
        .unwrap()
        .as_secs();

    match hello {
        Ok(hello) => info!("Run Long returned after {} seconds: {:?}", seconds, hello),
        Err(e) => error!(":( {:?}", (e)),
    }
    let start = time::SystemTime::now();
    let hello = client
        .do_something2(tarpc::context::current(), BackendActionRequest::StartLong)
        .await;
    let seconds = (time::SystemTime::now().duration_since(start))
        .unwrap()
        .as_secs();

    match hello {
        Ok(hello) => info!("Start Long returned after {} seconds: {:?}", seconds, hello),
        Err(e) => info!(":( {:?}", (e)),
    }
}

pub async fn create_client(ip: IpAddr, port: u16) -> Result<GDriverServiceClient> {
    let server_addr = (ip, port);
    let transport = tarpc::serde_transport::tcp::connect(&server_addr, Json::default)
        .await
        .map_err(|e| {
            info!("Could not connect to backend. Please make sure it is started before this app.");
            e
        })?;
    let service = GDriverServiceClient::new(client::Config::default(), transport);
    let client = service.spawn();
    let _ = ping(&client).await;
    Ok(client)
}
