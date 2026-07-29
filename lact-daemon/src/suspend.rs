use crate::server::handler::Handler;
use futures::StreamExt;
use std::time::Duration;
use tracing::{error, info};
use zbus::{Connection, Proxy};

const SUSPEND_EVENT_COLLECT_DURATION: Duration = Duration::from_secs(3);

pub async fn listen_events(handler: Handler) {
    match connect_proxy().await {
        // Note: despite the name, the events get triggered both on suspend and resume
        Ok(proxy) => match proxy.receive_signal("PrepareForSleep").await {
            Ok(mut stream) => {
                while stream.next().await.is_some() {
                    info!("suspend/resume event detected, queueing config reload");
                    handler
                        .notify_reload_gpus(SUSPEND_EVENT_COLLECT_DURATION)
                        .await;
                }
            }
            Err(err) => error!("could not subscribe to suspend events: {err:#}"),
        },
        Err(err) => {
            error!("could not connect to dbus proxy: {err:#}");
        }
    }
    error!("suspend/resume events will not be handled.");
}

async fn connect_proxy() -> anyhow::Result<Proxy<'static>> {
    let conn = Box::pin(Connection::system()).await?;
    let proxy = Proxy::new_owned(
        conn,
        "org.freedesktop.login1",
        "/org/freedesktop/login1",
        "org.freedesktop.login1.Manager",
    )
    .await?;
    Ok(proxy)
}
