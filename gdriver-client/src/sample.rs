use gdriver_common::config::CONFIGURATION;

use super::*;
pub async fn start() -> Result<()> {
    println!("Hello, world!");

    let name = "test1".to_string();
    let config = &CONFIGURATION;
    let client: WorldClient = create_client(config.ip, config.port).await?;

    let hello = client
        .hello(tarpc::context::current(), name.to_string())
        .await;

    match hello {
        Ok(hello) => println!("{hello:?}"),
        Err(e) => println!("{:?}", (e)),
    }
    Ok(())
}
pub async fn create_client(ip: IpAddr, port: u16) -> Result<WorldClient> {
    let server_addr = (ip, port);
    let transport = tarpc::serde_transport::tcp::connect(&server_addr, Json::default)
        .await
        .map_err(|e| {
            println!("Could not connect");
            e
        })?;
    let var_name = WorldClient::new(client::Config::default(), transport);
    let client = var_name.spawn();
    Ok(client)
}
