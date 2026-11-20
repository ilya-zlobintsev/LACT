//! Detects whether the desktop environment is able to display a tray icon.
//!
//! GNOME does not implement StatusNotifierItem on its own: without the AppIndicator extension
//! there is no StatusNotifierWatcher host on the session bus, so the item registered by
//! `lact tray` exists but nothing renders it. The GUI uses the checks here to tell the user that
//! the icon is missing because of the desktop, instead of silently showing nothing.

use gtk::{
    gio,
    glib::{self, variant::ToVariant as _},
};

/// GNOME Shell extension which adds a StatusNotifierItem host and makes tray icons visible.
pub const GNOME_EXTENSION_URL: &str =
    "https://extensions.gnome.org/extension/615/appindicator-support/";

const WATCHER_BUS_NAME: &str = "org.kde.StatusNotifierWatcher";
const WATCHER_INTERFACE: &str = "org.kde.StatusNotifierWatcher";
const WATCHER_PATH: &str = "/StatusNotifierWatcher";
const HOST_REGISTERED_PROPERTY: &str = "IsStatusNotifierHostRegistered";
const PROPERTIES_INTERFACE: &str = "org.freedesktop.DBus.Properties";
const DBUS_INTERFACE: &str = "org.freedesktop.DBus";
const DBUS_PATH: &str = "/org/freedesktop/DBus";
const DBUS_TIMEOUT: i32 = 1000;

/// Whether the current desktop can display a tray icon.
///
/// The icon is shown by a StatusNotifierItem host, so a session without one renders nothing.
pub fn is_supported() -> bool {
    let Ok(connection) = gio::bus_get_sync(gio::BusType::Session, gio::Cancellable::NONE) else {
        return false;
    };

    // Without a watcher there is no host to display the icon, which is the case on GNOME
    // without the AppIndicator extension.
    if !name_has_owner(&connection) {
        return false;
    }

    // A host which does not expose this property still displays icons, so a failed read must
    // not produce a warning on a working desktop.
    host_registered(&connection).unwrap_or(true)
}

/// Whether the current desktop is GNOME.
pub fn is_gnome() -> bool {
    is_gnome_desktop(std::env::var("XDG_CURRENT_DESKTOP").ok().as_deref())
}

/// Checks the value of `XDG_CURRENT_DESKTOP`, which is a `:`-separated list of desktop names.
fn is_gnome_desktop(current_desktop: Option<&str>) -> bool {
    current_desktop.is_some_and(|desktop| {
        desktop
            .split(':')
            .any(|name| name.eq_ignore_ascii_case("GNOME"))
    })
}

/// Whether the StatusNotifierWatcher name is owned on the session bus.
fn name_has_owner(connection: &gio::DBusConnection) -> bool {
    // Only fails for an invalid type signature, which a literal cannot be.
    let Ok(reply_type) = glib::VariantTy::new("(b)") else {
        return false;
    };

    let reply = connection
        .call_sync(
            Some(DBUS_INTERFACE),
            DBUS_PATH,
            DBUS_INTERFACE,
            "NameHasOwner",
            Some(&(WATCHER_BUS_NAME,).to_variant()),
            Some(reply_type),
            gio::DBusCallFlags::NONE,
            DBUS_TIMEOUT,
            gio::Cancellable::NONE,
        )
        .ok()
        .and_then(|reply| reply.get::<(bool,)>());

    reply.is_some_and(|(has_owner,)| has_owner)
}

/// Reads whether the watcher has a registered host, `None` when the property is unavailable.
fn host_registered(connection: &gio::DBusConnection) -> Option<bool> {
    let reply = connection
        .call_sync(
            Some(WATCHER_BUS_NAME),
            WATCHER_PATH,
            PROPERTIES_INTERFACE,
            "Get",
            Some(&(WATCHER_INTERFACE, HOST_REGISTERED_PROPERTY).to_variant()),
            Some(glib::VariantTy::new("(v)").ok()?),
            gio::DBusCallFlags::NONE,
            DBUS_TIMEOUT,
            gio::Cancellable::NONE,
        )
        .ok()?;

    let (value,) = reply.get::<(glib::Variant,)>()?;

    value.get::<bool>()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_gnome_desktop() {
        assert!(is_gnome_desktop(Some("GNOME")));
        assert!(is_gnome_desktop(Some("ubuntu:GNOME")));
        assert!(is_gnome_desktop(Some("gnome")));
    }

    #[test]
    fn does_not_detect_other_desktops() {
        assert!(!is_gnome_desktop(Some("KDE")));
        assert!(!is_gnome_desktop(Some("")));
        assert!(!is_gnome_desktop(None));
    }
}
