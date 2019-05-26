#[macro_use] extern crate log;
use std::env;
use kube::{
    api::{Api, Informer, WatchEvent},
    client::APIClient,
    config,
};
use k8s_openapi::api::core::v1::{PodSpec, PodStatus};

fn main() -> Result<(), failure::Error> {
    env::set_var("RUST_LOG", "info,kube=trace");
    env_logger::init();
    let config = config::load_kube_config().expect("failed to load kubeconfig");
    let client = APIClient::new(config);
    let namespace = env::var("NAMESPACE").unwrap_or("kube-system".into());

    let resource = Api::v1Pod().within(&namespace);
    let inf = Informer::new(client.clone(), resource).init()?;

    // Here we both poll and reconcile based on events from the main thread
    // If you run this next to actix-web (say), spawn a thread and pass `inf` as app state
    loop {
        inf.poll()?;

        // Handle events one by one, draining the informer
        while let Some(event) = inf.pop() {
            handle_node(&client, event)?;
        }
    }
}

// This function lets the app handle an event from kube
fn handle_node(_c: &APIClient, ev: WatchEvent<PodSpec, PodStatus>) -> Result<(), failure::Error> {
    match ev {
        WatchEvent::Added(o) => {
            let containers = o.spec.containers.into_iter().map(|c| c.name).collect::<Vec<_>>();
            info!("Added Pod: {} (containers={:?})", o.metadata.name, containers);
        },
        WatchEvent::Modified(o) => {
            let phase = o.status.unwrap().phase.unwrap();
            info!("Modified Pod: {} (phase={})", o.metadata.name, phase);
        },
        WatchEvent::Deleted(o) => {
            info!("Deleted Pod: {}", o.metadata.name);
        },
        WatchEvent::Error(e) => {
            warn!("Error event: {:?}", e);
        }
    }
    Ok(())
}
