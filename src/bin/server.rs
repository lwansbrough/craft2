use std::{net::SocketAddr, sync::Arc};

use actix::{Actor, System, Arbiter};
use actix_telepathy::Cluster;
use azure_core::HttpClient;
use azure_data_tables::clients::{AsTableServiceClient, AsTableClient, AsPartitionKeyClient, AsEntityClient};
use azure_storage::clients::{StorageAccountClient, AsStorageClient};
use bevy::{
    diagnostic::{LogDiagnosticsPlugin, DiagnosticsPlugin},
    ecs::prelude::*,
    prelude::{App},
    MinimalPlugins, transform::TransformPlugin, asset::AssetPlugin, hierarchy::HierarchyPlugin,
};
use craft2::{VoxelVolumePlugin, zoning::MyMessage};
use actix_rt;
use tokio::runtime::Runtime;
use uuid::Uuid;
use futures::stream::StreamExt;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct Server {
    #[serde(rename = "PartitionKey")]
    pub cluster: String,
    #[serde(rename = "RowKey")]
    pub id: String,
    pub address: String,
}

// fn main() {
//     let _sys = System::new();

//     let a1 = Arbiter::new();
//     let a2 = Arbiter::new();

//     a1.spawn(async { run_server().await });
//     a2.spawn(async {
//         App::new()
//             .add_plugins(MinimalPlugins)
//             .add_plugin(TransformPlugin)
//             .add_plugin(AssetPlugin)
//             .add_plugin(HierarchyPlugin)
//             .add_plugin(DiagnosticsPlugin)
//             .add_plugin(LogDiagnosticsPlugin::default())
//             .add_plugin(VoxelVolumePlugin)
//             .add_startup_system(setup)
//             .run();
//     });

//     a1.join().unwrap();
// }

#[actix_rt::main]
pub async fn main() {
    run_server().await;

    println!("server setup");

    App::new()
        .add_plugins(MinimalPlugins)
        .add_plugin(TransformPlugin)
        .add_plugin(AssetPlugin)
        .add_plugin(HierarchyPlugin)
        .add_plugin(DiagnosticsPlugin)
        .add_plugin(LogDiagnosticsPlugin::default())
        .add_plugin(VoxelVolumePlugin)
        .add_startup_system(setup)
        .run();
}

fn setup(
    mut commands: Commands
) {
    
}

async fn run_server() {
    let id = Uuid::new_v4();
    let address = format!("127.0.0.1:{}", portpicker::pick_unused_port().unwrap());

    let mut server = Server {
        cluster: "main".to_owned(),
        id: id.to_string().to_owned(),
        address: address.to_owned(),
    };
    
    let http_client = azure_core::new_http_client();
    let storage_account_client = StorageAccountClient::new_access_key(http_client.clone(), "lochland", "tBZHPndbFalLk1FewKuI1Ju/4MDxGVO8bbGO7IbYn+eqMxAxtAwWbIgBcprsepVGN8b7r15G7S+I+ASt7NLoRA==");
    let table_service = storage_account_client
        .as_storage_client()
        .as_table_service_client()
        .unwrap();

    let table_client = table_service.as_table_client("servers");
    let entity_client = table_client
        .as_partition_key_client("main".to_owned())
        .as_entity_client(id.to_string())
        .unwrap();

    let response = entity_client
        .insert_or_replace()
        .execute(&server)
        .await
        .unwrap();

    let mut stream = Box::pin(
        table_client
            .query()
            .stream::<Server>(),
    );

    let mut seed_nodes: Vec<SocketAddr> = Vec::new();

    while let Some(response) = stream.next().await {
        for entity in response.unwrap().entities.iter() {
            if entity.address == address {
                continue;
            }
            seed_nodes.push(entity.address.parse().unwrap());
            println!("{:?}", seed_nodes.last().unwrap());
        }
    }

    let own_addr: SocketAddr = address.parse().unwrap();

    let _actor = craft2::zoning::MyActor { state: 0 }.start();
    let _cluster = Cluster::new(own_addr, seed_nodes);

    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
}