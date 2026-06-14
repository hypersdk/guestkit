// SPDX-License-Identifier: Apache-2.0
//! org.freedesktop.login1 D-Bus collector.

use anyhow::{Context, Result};
use guestkit_agent_protocol::{LoggedInUser, LoginState, ShutdownInhibitor};
use zbus::blocking::Connection;
use zbus::zvariant::OwnedValue;

pub fn collect_login_state() -> Result<LoginState> {
    let conn = Connection::system().context("connect to system dbus")?;
    let manager = zbus::blocking::Proxy::new(
        &conn,
        "org.freedesktop.login1",
        "/org/freedesktop/login1",
        "org.freedesktop.login1.Manager",
    )?;

    let idle_hint: bool = manager
        .get_property("IdleHint")
        .unwrap_or(Ok(false))
        .unwrap_or(false);

    let sessions_raw: Vec<OwnedValue> = manager.call("ListSessions", &()).unwrap_or_default();
    let mut logged_in_users = Vec::new();
    for entry in sessions_raw {
        if let Ok(row) = entry.downcast_ref::<zbus::zvariant::Structure>() {
            let fields = row.fields();
            if fields.len() < 4 {
                continue;
            }
            let user_name = fields
                .get(2)
                .and_then(|v| v.downcast_ref::<String>())
                .map(|s| s.clone())
                .unwrap_or_default();
            let seat = fields
                .get(3)
                .and_then(|v| v.downcast_ref::<String>())
                .map(|s| s.clone())
                .unwrap_or_default();
            logged_in_users.push(LoggedInUser {
                name: user_name,
                seat,
                session_type: "unknown".into(),
                active: true,
            });
        }
    }

    let inhibitors_raw: Vec<OwnedValue> = manager.call("ListInhibitors", &()).unwrap_or_default();
    let mut inhibitors = Vec::new();
    for entry in inhibitors_raw {
        if let Ok(row) = entry.downcast_ref::<zbus::zvariant::Structure>() {
            let fields = row.fields();
            if fields.len() < 6 {
                continue;
            }
            inhibitors.push(ShutdownInhibitor {
                what: fields
                    .get(2)
                    .and_then(|v| v.downcast_ref::<String>())
                    .map(|s| s.clone())
                    .unwrap_or_default(),
                who: fields
                    .get(3)
                    .and_then(|v| v.downcast_ref::<String>())
                    .map(|s| s.clone())
                    .unwrap_or_default(),
                why: fields
                    .get(4)
                    .and_then(|v| v.downcast_ref::<String>())
                    .map(|s| s.clone())
                    .unwrap_or_default(),
                mode: fields
                    .get(5)
                    .and_then(|v| v.downcast_ref::<String>())
                    .map(|s| s.clone())
                    .unwrap_or_default(),
            });
        }
    }

    Ok(LoginState {
        logged_in_users,
        inhibitors,
        idle_hint,
    })
}
