use crate::ledger::{is_valid_ledger, LedgerWallet};
use log::*;
use parking_lot::{Mutex, RwLock};
use solana_sdk::{pubkey::Pubkey, signature::Signature, transaction::Transaction};
use std::{
    fmt,
    str::FromStr,
    sync::Arc,
    time::{Duration, Instant},
};
use thiserror::Error;

const HID_GLOBAL_USAGE_PAGE: u16 = 0xFF00;
const HID_USB_DEVICE_CLASS: u8 = 0;

/// Remote wallet error.
#[derive(Error, Debug)]
pub enum RemoteWalletError {
    #[error("hidapi error")]
    Hid(#[from] hidapi::HidError),

    #[error("device type mismatch")]
    DeviceTypeMismatch,

    #[error("device with non-supported product ID or vendor ID was detected")]
    InvalidDevice,

    #[error("invalid derivation path: {0}")]
    InvalidDerivationPath(String),

    #[error("invalid path: {0}")]
    InvalidPath(String),

    #[error("no device found")]
    NoDeviceFound,

    #[error("protocol error: {0}")]
    Protocol(&'static str),

    #[error("pubkey not found for given address")]
    PubkeyNotFound,

    #[error("operation has been cancelled")]
    UserCancel,
}

/// Collection of conntected RemoteWallets
pub struct RemoteWalletManager {
    usb: Arc<Mutex<hidapi::HidApi>>,
    devices: RwLock<Vec<Device>>,
}

impl RemoteWalletManager {
    /// Create a new instance.
    pub fn new(usb: Arc<Mutex<hidapi::HidApi>>) -> Arc<Self> {
        Arc::new(Self {
            usb,
            devices: RwLock::new(Vec::new()),
        })
    }

    /// Repopulate device list
    /// Note: this method iterates over and updates all devices
    pub fn update_devices(&self) -> Result<usize, RemoteWalletError> {
        let mut usb = self.usb.lock();
        usb.refresh_devices()?;
        let devices = usb.devices();
        let num_prev_devices = self.devices.read().len();

        let detected_devices = devices
            .iter()
            .filter(|&device_info| {
                is_valid_hid_device(device_info.usage_page, device_info.interface_number)
            })
            .fold(Vec::new(), |mut v, device_info| {
                if is_valid_ledger(device_info.vendor_id, device_info.product_id) {
                    match usb.open_path(&device_info.path) {
                        Ok(device) => {
                            let ledger = LedgerWallet::new(device);
                            if let Ok(info) = ledger.read_device(&device_info) {
                                let path = device_info.path.to_str().unwrap().to_string();
                                trace!("Found device: {:?}", info);
                                v.push(Device {
                                    path,
                                    info,
                                    wallet_type: RemoteWalletType::Ledger(Arc::new(ledger)),
                                })
                            }
                        }
                        Err(e) => error!("Error connecting to ledger device to read info: {}", e),
                    }
                }
                v
            });

        let num_curr_devices = detected_devices.len();
        *self.devices.write() = detected_devices;

        Ok(num_curr_devices - num_prev_devices)
    }

    /// List connected and acknowledged wallets
    pub fn list_devices(&self) -> Vec<RemoteWalletInfo> {
        self.devices.read().iter().map(|d| d.info.clone()).collect()
    }

    /// Get a particular wallet
    #[allow(unreachable_patterns)]
    pub fn get_ledger(&self, pubkey: &Pubkey) -> Result<Arc<LedgerWallet>, RemoteWalletError> {
        self.devices
            .read()
            .iter()
            .find(|device| &device.info.pubkey == pubkey)
            .ok_or(RemoteWalletError::PubkeyNotFound)
            .and_then(|device| match &device.wallet_type {
                RemoteWalletType::Ledger(ledger) => Ok(ledger.clone()),
                _ => Err(RemoteWalletError::DeviceTypeMismatch),
            })
    }

    /// Get wallet info.
    pub fn get_wallet_info(&self, pubkey: &Pubkey) -> Option<RemoteWalletInfo> {
        self.devices
            .read()
            .iter()
            .find(|d| &d.info.pubkey == pubkey)
            .map(|d| d.info.clone())
    }

    /// Update devices in maximum `max_polling_duration` if it doesn't succeed
    pub fn try_connect_polling(&self, max_polling_duration: &Duration) -> bool {
        let start_time = Instant::now();
        while start_time.elapsed() <= *max_polling_duration {
            if let Ok(num_devices) = self.update_devices() {
                let plural = if num_devices == 1 { "" } else { "s" };
                trace!("{} Remote Wallet{} found", num_devices, plural);
                return true;
            }
        }
        false
    }
}

/// `RemoteWallet` trait
pub trait RemoteWallet {
    /// Parse device info and get device base pubkey
    fn read_device(
        &self,
        dev_info: &hidapi::HidDeviceInfo,
    ) -> Result<RemoteWalletInfo, RemoteWalletError>;

    /// Get solana pubkey from a RemoteWallet
    fn get_pubkey(&self, derivation: DerivationPath) -> Result<Pubkey, RemoteWalletError>;

    /// Sign transaction data with wallet managing pubkey at derivation path m/44'/501'/<account>'/<change>'.
    fn sign_transaction(
        &self,
        derivation: DerivationPath,
        transaction: Transaction,
    ) -> Result<Signature, RemoteWalletError>;
}

/// `RemoteWallet` device
#[derive(Debug)]
pub struct Device {
    pub(crate) path: String,
    pub(crate) info: RemoteWalletInfo,
    pub wallet_type: RemoteWalletType,
}

/// Remote wallet convenience enum to hold various wallet types
#[derive(Debug)]
pub enum RemoteWalletType {
    Ledger(Arc<LedgerWallet>),
}

/// Remote wallet information.
#[derive(Debug, Default, Clone)]
pub struct RemoteWalletInfo {
    /// RemoteWallet device model
    pub model: String,
    /// RemoteWallet device manufacturer
    pub manufacturer: String,
    /// RemoteWallet device serial number
    pub serial: String,
    /// Base pubkey of device at Solana derivation path
    pub pubkey: Pubkey,
}

impl RemoteWalletInfo {
    pub fn parse_path(mut path: String) -> Result<(Self, DerivationPath), RemoteWalletError> {
        let mut path = path.split_off(6);
        if path.ends_with('/') {
            path.pop();
        }
        let mut parts = path.split('/');
        let mut wallet_info = RemoteWalletInfo::default();
        let manufacturer = parts.next().unwrap();
        wallet_info.manufacturer = manufacturer.to_string();
        wallet_info.model = parts.next().unwrap_or("").to_string();
        wallet_info.pubkey = parts
            .next()
            .and_then(|pubkey_str| Pubkey::from_str(pubkey_str).ok())
            .unwrap_or_default();

        let mut derivation_path = DerivationPath::default();
        if let Some(purpose) = parts.next() {
            if purpose.replace("'", "") != "44" {
                return Err(RemoteWalletError::InvalidDerivationPath(format!(
                    "Incorrect purpose number, found: {}, must be 44",
                    purpose
                )));
            }
            if let Some(coin) = parts.next() {
                if coin.replace("'", "") != "501" {
                    return Err(RemoteWalletError::InvalidDerivationPath(format!(
                        "Incorrect coin number, found: {}, must be 501",
                        coin
                    )));
                }
                if let Some(account) = parts.next() {
                    derivation_path.account = account.replace("'", "").parse::<u16>().unwrap();
                    derivation_path.change = parts
                        .next()
                        .and_then(|change| change.replace("'", "").parse::<u16>().ok());
                }
            } else {
                return Err(RemoteWalletError::InvalidDerivationPath(
                    "Derivation path too short, missing coin number".to_string(),
                ));
            }
        }
        Ok((wallet_info, derivation_path))
    }

    pub fn get_pretty_path(&self) -> String {
        format!(
            "usb://{}/{}/{:?}",
            self.manufacturer, self.model, self.pubkey,
        )
    }

    pub(crate) fn matches(&self, other: &Self) -> bool {
        self.manufacturer == other.manufacturer
            && (self.model == other.model || self.model == "" || other.model == "")
            && (self.pubkey == other.pubkey
                || self.pubkey == Pubkey::default()
                || other.pubkey == Pubkey::default())
    }
}

#[derive(Default, PartialEq, Clone)]
pub struct DerivationPath {
    pub account: u16,
    pub change: Option<u16>,
}

impl fmt::Debug for DerivationPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let change = if let Some(change) = self.change {
            format!("/{:?}'", change)
        } else {
            "".to_string()
        };
        write!(f, "m/44'/501'/{:?}'{}", self.account, change)
    }
}

/// Helper to determine if a device is a valid HID
pub fn is_valid_hid_device(usage_page: u16, interface_number: i32) -> bool {
    usage_page == HID_GLOBAL_USAGE_PAGE || interface_number == HID_USB_DEVICE_CLASS as i32
}

/// Helper to initialize hidapi and RemoteWalletManager
pub fn initialize_wallet_manager() -> Arc<RemoteWalletManager> {
    let hidapi = Arc::new(Mutex::new(hidapi::HidApi::new().unwrap()));
    RemoteWalletManager::new(hidapi)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_path() {
        let pubkey = Pubkey::new_rand();
        let (wallet_info, derivation_path) =
            RemoteWalletInfo::parse_path(format!("usb://ledger/nano-s/{:?}/44/501/1/2", pubkey))
                .unwrap();
        assert!(wallet_info.matches(&RemoteWalletInfo {
            model: "nano-s".to_string(),
            manufacturer: "ledger".to_string(),
            serial: "".to_string(),
            pubkey,
        }));
        assert_eq!(
            derivation_path,
            DerivationPath {
                account: 1,
                change: Some(2),
            }
        );
        let (wallet_info, derivation_path) = RemoteWalletInfo::parse_path(format!(
            "usb://ledger/nano-s/{:?}/44'/501'/1'/2'",
            pubkey
        ))
        .unwrap();
        assert!(wallet_info.matches(&RemoteWalletInfo {
            model: "nano-s".to_string(),
            manufacturer: "ledger".to_string(),
            serial: "".to_string(),
            pubkey,
        }));
        assert_eq!(
            derivation_path,
            DerivationPath {
                account: 1,
                change: Some(2),
            }
        );

        assert!(RemoteWalletInfo::parse_path(format!(
            "usb://ledger/nano-s/{:?}/43/501/1/2",
            pubkey
        ))
        .is_err());
        assert!(RemoteWalletInfo::parse_path(format!(
            "usb://ledger/nano-s/{:?}/44/500/1/2",
            pubkey
        ))
        .is_err());
    }

    #[test]
    fn test_remote_wallet_info_matches() {
        let pubkey = Pubkey::new_rand();
        let info = RemoteWalletInfo {
            manufacturer: "Ledger".to_string(),
            model: "Nano S".to_string(),
            serial: "0001".to_string(),
            pubkey: pubkey.clone(),
        };
        let mut test_info = RemoteWalletInfo::default();
        test_info.manufacturer = "Not Ledger".to_string();
        assert!(!info.matches(&test_info));
        test_info.manufacturer = "Ledger".to_string();
        assert!(info.matches(&test_info));
        test_info.model = "Other".to_string();
        assert!(!info.matches(&test_info));
        test_info.model = "Nano S".to_string();
        assert!(info.matches(&test_info));
        let another_pubkey = Pubkey::new_rand();
        test_info.pubkey = another_pubkey;
        assert!(!info.matches(&test_info));
        test_info.pubkey = pubkey;
        assert!(info.matches(&test_info));
    }
}
