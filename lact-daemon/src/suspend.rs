use crate::server::handler::Handler;
use futures::StreamExt;
use tracing::{error, info};
use zbus::{Connection, Proxy};

pub async fn listen_events(handler: Handler) {
    match connect_proxy().await {
        // Note: despite the name, the events get triggered both on suspend and resume
        Ok(proxy) => match proxy.receive_signal("PrepareForSleep").await {
            Ok(mut stream) => {
                while stream.next().await.is_some() {
                    info!("suspend/resume event detected, reloading config");
                    if let Err(err) = handler.apply_current_config().await {
                        error!("could not reapply config: {err:#}");
                    }
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
