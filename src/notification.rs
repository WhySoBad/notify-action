use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};

use anyhow::{Context, bail};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::{Receiver, Sender, channel};
use zbus::object_server::InterfaceRef;
use zbus::zvariant::{OwnedValue, Type};
use zbus::{Connection, interface};

pub type NotificationId = u32;

#[expect(unused)]
#[derive(Debug)]
pub struct Notification {
    pub app_name: String,
    pub id: NotificationId,
    pub icon: String,
    pub summary: String,
    pub body: String,
    pub actions: Vec<(String, String)>,
    pub timeout: i32,
    pub urgency: Urgency,
}

#[derive(Deserialize, Type, Debug, Clone, OwnedValue, PartialEq)]
#[repr(u8)]
#[serde(rename_all = "kebab-case")]
pub enum Urgency {
    Low = 0,
    Normal = 1,
    Critical = 2,
}

#[derive(Debug, Type, Serialize, Deserialize)]
struct ServerInformation {
    pub name: &'static str,
    pub vendor: &'static str,
    pub version: &'static str,
    pub spec_version: &'static str,
}

const SERVER_INFORMATION: ServerInformation = ServerInformation {
    name: "notify-alert",
    vendor: "notifications",
    version: "0.1.0",
    spec_version: "1.2",
};

static NOTIFICATION_COUNT: AtomicU32 = AtomicU32::new(1);

pub struct Notifications {
    tx: Sender<Notification>,
}

impl Notifications {
    /// Initialize the dbus notifications service.
    pub async fn initialize()
    -> anyhow::Result<(InterfaceRef<Notifications>, Receiver<Notification>)> {
        let (tx, rx) = channel(10);

        let connection = Connection::session().await.context("unable to connect to session bus")?;

        match connection.object_server().at("/org/freedesktop/Notifications", Self { tx }).await {
            Ok(false) => bail!("unable to bind dbus notifications: destination is already bound"),
            Err(err) => bail!("unable to bind dbus notifications: {err}"),
            Ok(true) => connection
                .request_name("org.freedesktop.Notifications")
                .await
                .context("unable to required notifications name")?,
        };

        let interface = connection
            .object_server()
            .interface::<_, Notifications>("/org/freedesktop/Notifications")
            .await
            .context("unable to acquire interface")?;

        Ok((interface, rx))
    }
}

#[interface(name = "org.freedesktop.Notifications")]
impl Notifications {
    fn get_capabilities(&self) -> Vec<&'static str> {
        vec![
            "body",
            "icon-static",
            "actions",
            "x-canonical-private-synchronous",
            "x-dunst-stack-tag",
        ]
    }

    fn get_server_information(&self) -> ServerInformation {
        SERVER_INFORMATION
    }

    #[allow(clippy::too_many_arguments)]
    async fn notify(
        &mut self,
        app_name: &str,
        replaced_id: u32,
        icon: &str,
        summary: &str,
        body: &str,
        actions: Vec<&str>,
        mut hints: HashMap<&str, OwnedValue>,
        timeout: i32,
    ) -> zbus::fdo::Result<u32> {
        let id = if replaced_id == 0 {
            NOTIFICATION_COUNT.fetch_add(1, Ordering::Relaxed)
        } else {
            replaced_id
        };

        let urgency =
            hints.remove("urgency").and_then(|v| v.try_into().ok()).unwrap_or(Urgency::Low);

        let notification = Notification {
            app_name: app_name.to_string(),
            body: body.to_string(),
            summary: summary.to_string(),
            icon: icon.to_string(),
            actions: actions
                .chunks(2)
                .map(|chunk| {
                    (
                        chunk.first().unwrap_or(&"").to_string(),
                        chunk.last().unwrap_or(&"").to_string(),
                    )
                })
                .collect(),
            id,
            timeout,
            urgency,
        };

        log::debug!("notification = {notification:?}");

        if let Err(err) = self.tx.try_send(notification) {
            log::error!("unable to send notification through channel: {err}");
        };

        Ok(id)
    }
}
