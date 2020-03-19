use k8s_openapi::{
    api::{
        batch::{
            v1::JobSpec,
            v1beta1::{CronJob, CronJobSpec, JobTemplateSpec},
        },
        core::v1::{Container, PodSpec, PodTemplateSpec},
    },
    apimachinery::pkg::apis::meta::v1::ObjectMeta,
};
use kube::{api::PostParams, client::APIClient, config, Api};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    std::env::set_var("RUST_LOG", "info,kube=debug");
    env_logger::init();
    let config = config::load_kube_config().await?;
    let client = APIClient::new(config);
    let namespace = std::env::var("NAMESPACE").unwrap_or("default".into());

    let api = Api::<CronJob>::namespaced(client, &namespace);
    let jobname = "blagger".to_string();
    let cron = "0 * * * *";

    api.create(&PostParams::default(), &CronJob {
        metadata: Some(ObjectMeta {
            name: Some(jobname),
            ..Default::default()
        }),
        spec: Some(CronJobSpec {
            schedule: cron.into(),
            job_template: JobTemplateSpec {
                spec: Some(JobSpec {
                    template: PodTemplateSpec {
                        spec: Some(PodSpec {
                            containers: vec![Container {
                                name: "hello".into(),
                                image: Some("busybox".into()),
                                args: Some(vec!["/bin/sh".into(), "-c".into(), "date; echo Hello".into()]),
                                ..Default::default()
                            }],
                            restart_policy: Some("OnFailure".into()),
                            ..Default::default()
                        }),
                        metadata: None,
                    },
                    ..Default::default()
                }),
                metadata: None,
            },
            ..Default::default()
        }),
        status: None,
    })
    .await?;

    Ok(())
}
