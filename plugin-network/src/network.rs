use glib::translate::FromGlib;

use async_std::task;
use futures_channel::oneshot;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;
use std::time::{Duration, SystemTime};

use anyhow::{anyhow, Context, Result};
use nm::*;

static SCAN_INTERVAL_MS: u64 = 500;
static SCAN_TOTAL_WAIT: u64 = 3;
static SCAN_BETWEEN: i64 = 30;

/// NetworkManager Simplified API
#[derive(Debug)]
pub struct Manager {
    client: Client,
    wifi: DeviceWifi,
    timeout: Duration,
}

/// AccessPoint Information
#[derive(Debug)]
pub struct AccessPoint {
    pub in_use: bool,
    pub ssid: String,
    pub rate: u32,
    pub signal: u8,
    pub security: String,
    pub is_active: bool,
    pub connection: Option<Connection>,
}

// SETTING_WIRELESS_MODE
// SETTING_IP4_CONFIG_METHOD_AUTO

/// Generate a NEW Connection to Use AccessPoint
fn new_conn(ap: &AccessPoint, password: Option<&str>) -> Result<SimpleConnection> {
    let connection = SimpleConnection::new();

    // configure generate connection settings
    let s_connection = SettingConnection::new();
    s_connection.set_type(Some(&SETTING_WIRELESS_SETTING_NAME));
    s_connection.set_id(Some(&ap.ssid));
    s_connection.set_autoconnect(false);
    // s_connection.set_interface_name(interface);
    connection.add_setting(s_connection);

    // configure wireless settings
    let s_wireless = SettingWireless::new();
    s_wireless.set_ssid(Some(&(ap.ssid.as_bytes().into())));
    // s_wireless.set_band(Some("bg"));
    // s_wireless.set_hidden(false);
    // s_wireless.set_mode(Some(&SETTING_WIRELESS_MODE));
    connection.add_setting(s_wireless);

    // configure login settings
    if let Some(password) = password {
        //TODO: potentially determine key-mgmt based on ap-security
        let s_wireless_security = SettingWirelessSecurity::new();
        s_wireless_security.set_key_mgmt(Some("wpa-psk"));
        s_wireless_security.set_psk(Some(password));
        connection.add_setting(s_wireless_security);
    }

    // assume DHCP Assignment
    let s_ip4 = SettingIP4Config::new();
    s_ip4.set_method(Some(&SETTING_IP4_CONFIG_METHOD_AUTO));
    connection.add_setting(s_ip4);

    Ok(connection)
}

/// Exit Security Section to Specify New Password
fn edit_conn(conn: &Connection, password: Option<&str>) -> Result<()> {
    let s_wireless_security = conn
        .setting_wireless_security()
        .unwrap_or_else(|| SettingWirelessSecurity::new());
    s_wireless_security.set_key_mgmt(Some("wpa-psk"));
    s_wireless_security.set_psk(password);
    conn.add_setting(s_wireless_security);
    Ok(())
}

/// Wait for Connection to Fail or Complete
async fn wait_conn(active: &ActiveConnection, timeout: Duration) -> Result<()> {
    // spawn communication channels
    let (sender, receiver) = oneshot::channel::<Result<()>>();
    let sender = Rc::new(RefCell::new(Some(sender)));
    // spawn state-callback
    active.connect_state_changed(move |active_connection, state, _| {
        let sender = sender.clone();
        let active_connection = active_connection.clone();
        glib::MainContext::ref_thread_default().spawn_local(async move {
            let state = unsafe { ActiveConnectionState::from_glib(state as _) };
            log::debug!("[Connect] Active connection state: {:?}", state);
            // generate send function
            let send = move |result| {
                let sender = sender.borrow_mut().take();
                if let Some(sender) = sender {
                    sender.send(result).expect("Sender Dropped");
                }
            };
            // handle connection state-changes
            match state {
                ActiveConnectionState::Activated => {
                    log::debug!("[Connect] Successfully activated");
                    return send(Ok(()));
                }
                ActiveConnectionState::Deactivated => {
                    log::debug!("[Connect] Connection deactivated");
                    match active_connection.connection() {
                        Some(remote_connection) => {
                            let result = remote_connection
                                .delete_future()
                                .await
                                .context("Failed to delete connection");
                            if result.is_err() {
                                return send(result);
                            }
                            return send(Err(anyhow!("Connection Failed (Deactivated)")));
                        }
                        None => {
                            return send(Err(anyhow!(
                                "Failed to get remote connection from active connection"
                            )))
                        }
                    }
                }
                _ => {}
            };
        });
    });
    // wait until state notification is done
    let result = async_std::future::timeout(timeout, receiver).await;
    match result {
        Ok(result) => match result {
            Ok(res) => res,
            Err(err) => Err(anyhow!("Connection Cancelled: {err:?}")),
        },
        Err(err) => Err(anyhow!("Timeout Reached: {err:?}")),
    }
}

impl Manager {
    /// Spawn new Wifi Manager Instance
    pub async fn new() -> Result<Self> {
        // get network-manager client
        let client = Client::new_future()
            .await
            .context("Failed to create NM Client")?;
        // get wifi device if any are available
        let device = client
            .devices()
            .into_iter()
            .filter(|d| d.device_type() == DeviceType::Wifi)
            .next()
            .ok_or_else(|| anyhow!("Cannot find a Wi-Fi device"))?;
        // access inner wifi-device object
        let wifi: DeviceWifi = device
            .downcast()
            .map_err(|_| anyhow!("Failed to Access Wi-Fi Device"))?;
        log::debug!("NetworkManager Connection Established");
        Ok(Self {
            client,
            wifi,
            timeout: Duration::from_secs(30),
        })
    }

    /// Update Manager Timeout and Return Self
    pub fn with_timeout(mut self, secs: u32) -> Self {
        self.timeout = Duration::from_secs(secs.into());
        self
    }

    /// Check if NetworkManager already scanned recently
    pub fn scanned_recently(&self) -> bool {
        let last_ms = self.wifi.last_scan();
        let now_ms = utils_get_timestamp_msec();
        let elapsed = (now_ms - last_ms) / 1000;
        last_ms > 0 && elapsed < SCAN_BETWEEN
    }

    /// Complete General Wifi-Scan
    pub async fn scan_wifi(&self) -> Result<()> {
        // request wifi-scan
        self.wifi
            .request_scan_future()
            .await
            .context("Failed to Request Wi-Fi Scan")?;
        // wait until access-points are collected
        let mut then = SystemTime::now();
        let mut current = self.wifi.access_points().len();
        loop {
            // wait interval for more access-points
            task::sleep(Duration::from_millis(SCAN_INTERVAL_MS)).await;
            // check if time has elapsed
            let now = SystemTime::now();
            let elapsed = now.duration_since(then)?;
            if elapsed.as_secs() > SCAN_TOTAL_WAIT {
                break;
            }
            // check if more access-points were discovered
            let found = self.wifi.access_points().len();
            if found > current {
                then = now;
                current = found;
            }
        }
        Ok(())
    }

    /// Retrieve Access-Point Information
    pub fn access_points(&self) -> Vec<AccessPoint> {
        let conns: Vec<Connection> = self
            .client
            .connections()
            .into_iter()
            .map(|c| c.upcast())
            .collect();
        let mut access: BTreeMap<String, AccessPoint> = BTreeMap::new();
        let active = self.wifi.active_access_point();
        for a in self.wifi.access_points() {
            // retrieve access-point information
            let rate = a.max_bitrate() / 1000;
            let signal = a.strength();
            let ssid = a
                .ssid()
                .map(|b| b.escape_ascii().to_string())
                .unwrap_or_else(|| "--".to_owned());
            // determine if connection-map should be updated
            let is_active = active
                .as_ref()
                .map(|b| a.bssid() == b.bssid())
                .unwrap_or(false);
            if !is_active {
                if let Some(point) = access.get(&ssid) {
                    if point.rate > rate {
                        continue;
                    }
                }
            }
            // build security-string
            let mut security = vec![];
            let wpa_flags = a.wpa_flags();
            let wpa2_flags = a.rsn_flags();
            if !wpa_flags.is_empty() {
                security.push("WPA1");
            }
            if !wpa2_flags.is_empty() {
                security.push("WPA2");
            }
            if wpa2_flags.intersects(_80211ApSecurityFlags::KEY_MGMT_802_1X) {
                security.push("802.1X");
            }
            if security.is_empty() {
                security.push("--");
            }
            // insert access-point
            access.insert(
                ssid.to_owned(),
                AccessPoint {
                    in_use: is_active,
                    ssid,
                    rate,
                    signal,
                    is_active,
                    security: security.join(" ").to_owned(),
                    connection: a.filter_connections(&conns).get(0).cloned(),
                },
            );
        }
        // move map values into vector and sort by signal-strength
        let mut points: Vec<AccessPoint> = access.into_values().collect();
        points.sort_by_key(|a| a.signal);
        points.reverse();
        points
    }

    /// Attempt to Authenticate and Activate Access-Point Connection
    pub async fn connect(&self, ap: &AccessPoint, password: Option<&str>) -> Result<()> {
        let device = self.wifi.clone().upcast::<Device>();
        match &ap.connection {
            Some(conn) => {
                edit_conn(conn, password)?;
                let active_conn = self
                    .client
                    .activate_connection_future(Some(conn), Some(&device), None)
                    .await
                    .context("Failed to activate existing connection")?;
                wait_conn(&active_conn, self.timeout).await?;
            }
            None => {
                let conn = new_conn(ap, password)?;
                let active_conn = self
                    .client
                    .add_and_activate_connection_future(Some(&conn), Some(&device), None)
                    .await
                    .context("Failed to add and activate connection")?;
                wait_conn(&active_conn, self.timeout).await?;
            }
        }
        Ok(())
    }
}
