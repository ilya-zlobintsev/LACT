use crate::server::handler::Handler;
use futures::StreamExt;
use std::time::Duration;
use tracing::{error, info};
use zbus::{Connection, proxy};

const SUSPEND_EVENT_COLLECT_DURATION: Duration = Duration::from_secs(2);

pub async fn listen_events(handler: Handler) {
    match connect_proxy().await {
        Ok(proxy) => match proxy.receive_prepare_for_sleep().await {
            Ok(mut stream) => {
                while let Some(value) = stream.next().await {
                    match value.args() {
                        Ok(args) => {
                            if !args.start {
                                info!("resume event detected, queueing config reload");
                                handler
                                    .notify_reload_gpus(SUSPEND_EVENT_COLLECT_DURATION)
                                    .await;
                            }
                        }
                        Err(err) => {
                            error!("could not get sleep events args: {err}");
                        }
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

async fn connect_proxy() -> anyhow::Result<LoginManagerProxy<'static>> {
    let conn = Box::pin(Connection::system()).await?;
    let proxy = LoginManagerProxy::new(&conn).await?;
    Ok(proxy)
}

#[proxy(
    interface = "org.freedesktop.login1.Manager",
    default_service = "org.freedesktop.login1",
    default_path = "/org/freedesktop/login1"
)]
pub trait LoginManager {
    // Note: despite the name, the events get triggered both on suspend and resume. `start` is false on resume
    #[zbus(signal)]
    fn prepare_for_sleep(&self, start: bool) -> zbus::Result<()>;
}
