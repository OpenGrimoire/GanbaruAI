//! Native credential storage for provider secrets.

use super::models::{
    ChatError, ChatErrorCode, ChatResult, CredentialReferenceId, ProviderInstanceConfig,
};
use serde::Serialize;
use std::fmt;

const CHAT_CREDENTIAL_SERVICE: &str = "com.ganbaru-ai.chat";
const AVAILABILITY_PROBE_REFERENCE: &str = "credential-store-availability-probe";

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CredentialStoreAvailability {
    Available,
    Unavailable,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CredentialStoreErrorCode {
    Unavailable,
    AccessDenied,
    InvalidReference,
    CorruptEntry,
    AmbiguousEntry,
    OperationFailed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CredentialStoreOperation {
    Probe,
    Read,
    Replace,
    Remove,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CredentialStoreError {
    pub code: CredentialStoreErrorCode,
    pub operation: CredentialStoreOperation,
}

impl CredentialStoreError {
    fn new(code: CredentialStoreErrorCode, operation: CredentialStoreOperation) -> Self {
        Self { code, operation }
    }
}

impl fmt::Display for CredentialStoreError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "credential store {:?} failed with {:?}",
            self.operation, self.code
        )
    }
}

impl std::error::Error for CredentialStoreError {}

#[derive(Eq, PartialEq)]
pub struct SecretValue(String);

impl SecretValue {
    pub fn new(value: impl Into<String>) -> Result<Self, CredentialStoreError> {
        let value = value.into();
        if value.is_empty() {
            return Err(CredentialStoreError::new(
                CredentialStoreErrorCode::OperationFailed,
                CredentialStoreOperation::Replace,
            ));
        }
        Ok(Self(value))
    }

    pub(crate) fn expose(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for SecretValue {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("SecretValue([REDACTED])")
    }
}

pub trait CredentialStore: Send + Sync {
    fn availability(&self) -> CredentialStoreAvailability;

    fn read(
        &self,
        reference: &CredentialReferenceId,
    ) -> Result<Option<SecretValue>, CredentialStoreError>;

    fn replace(
        &self,
        reference: &CredentialReferenceId,
        value: &SecretValue,
    ) -> Result<(), CredentialStoreError>;

    fn remove(&self, reference: &CredentialReferenceId) -> Result<bool, CredentialStoreError>;
}

pub fn materialize_provider_environment(
    configuration: &ProviderInstanceConfig,
    store: &impl CredentialStore,
) -> ChatResult<ProviderInstanceConfig> {
    let mut resolved = configuration.clone();
    for (name, value) in &configuration.environment {
        let Some(inherited_name) = value.strip_prefix("inherit:") else {
            continue;
        };
        if inherited_name != name {
            return Err(ChatError::new(
                ChatErrorCode::ConfigurationInvalid,
                format!("Inherited environment reference for {name} is invalid"),
                true,
            ));
        }
        let inherited = std::env::var(name).map_err(|_| {
            ChatError::new(
                ChatErrorCode::ConfigurationInvalid,
                format!("Inherited environment variable {name} is unavailable"),
                true,
            )
        })?;
        resolved.environment.insert(name.clone(), inherited);
    }
    for (name, reference) in &configuration.credential_references {
        if resolved.environment.contains_key(name) {
            return Err(ChatError::new(
                ChatErrorCode::ConfigurationInvalid,
                format!("Environment variable {name} has more than one configured source"),
                true,
            ));
        }
        let secret = store.read(reference).map_err(|_| {
            ChatError::new(
                ChatErrorCode::ConfigurationInvalid,
                format!("Stored credential for {name} could not be read"),
                true,
            )
        })?;
        let secret = secret.ok_or_else(|| {
            ChatError::new(
                ChatErrorCode::ConfigurationInvalid,
                format!("Stored credential for {name} is missing"),
                true,
            )
        })?;
        resolved
            .environment
            .insert(name.clone(), secret.expose().to_string());
    }
    resolved.credential_references.clear();
    Ok(resolved)
}

fn native_availability() -> CredentialStoreAvailability {
    match keyring::Entry::new(CHAT_CREDENTIAL_SERVICE, AVAILABILITY_PROBE_REFERENCE) {
        Ok(_) => CredentialStoreAvailability::Available,
        Err(_) => CredentialStoreAvailability::Unavailable,
    }
}

fn native_entry(
    reference: &CredentialReferenceId,
    operation: CredentialStoreOperation,
) -> Result<keyring::Entry, CredentialStoreError> {
    keyring::Entry::new(CHAT_CREDENTIAL_SERVICE, reference.as_str())
        .map_err(|error| map_keyring_error(&error, operation))
}

fn native_read(
    reference: &CredentialReferenceId,
) -> Result<Option<SecretValue>, CredentialStoreError> {
    let entry = native_entry(reference, CredentialStoreOperation::Read)?;
    match entry.get_password() {
        Ok(value) => SecretValue::new(value).map(Some),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(error) => Err(map_keyring_error(&error, CredentialStoreOperation::Read)),
    }
}

fn native_replace(
    reference: &CredentialReferenceId,
    value: &SecretValue,
) -> Result<(), CredentialStoreError> {
    let entry = native_entry(reference, CredentialStoreOperation::Replace)?;
    entry
        .set_password(value.expose())
        .map_err(|error| map_keyring_error(&error, CredentialStoreOperation::Replace))
}

fn native_remove(reference: &CredentialReferenceId) -> Result<bool, CredentialStoreError> {
    let entry = native_entry(reference, CredentialStoreOperation::Remove)?;
    match entry.delete_credential() {
        Ok(()) => Ok(true),
        Err(keyring::Error::NoEntry) => Ok(false),
        Err(error) => Err(map_keyring_error(&error, CredentialStoreOperation::Remove)),
    }
}

fn map_keyring_error(
    error: &keyring::Error,
    operation: CredentialStoreOperation,
) -> CredentialStoreError {
    let code = match error {
        keyring::Error::NoDefaultStore | keyring::Error::NotSupportedByStore(_) => {
            CredentialStoreErrorCode::Unavailable
        }
        keyring::Error::NoStorageAccess(_) => CredentialStoreErrorCode::AccessDenied,
        keyring::Error::Invalid(_, _) | keyring::Error::TooLong(_, _) => {
            CredentialStoreErrorCode::InvalidReference
        }
        keyring::Error::BadEncoding(_)
        | keyring::Error::BadDataFormat(_, _)
        | keyring::Error::BadStoreFormat(_) => CredentialStoreErrorCode::CorruptEntry,
        keyring::Error::Ambiguous(_) => CredentialStoreErrorCode::AmbiguousEntry,
        keyring::Error::PlatformFailure(_) | keyring::Error::NoEntry => {
            CredentialStoreErrorCode::OperationFailed
        }
        _ => CredentialStoreErrorCode::OperationFailed,
    };
    CredentialStoreError::new(code, operation)
}

macro_rules! native_credential_store {
    ($name:ident) => {
        #[derive(Clone, Copy, Debug, Default)]
        pub struct $name;

        impl CredentialStore for $name {
            fn availability(&self) -> CredentialStoreAvailability {
                native_availability()
            }

            fn read(
                &self,
                reference: &CredentialReferenceId,
            ) -> Result<Option<SecretValue>, CredentialStoreError> {
                native_read(reference)
            }

            fn replace(
                &self,
                reference: &CredentialReferenceId,
                value: &SecretValue,
            ) -> Result<(), CredentialStoreError> {
                native_replace(reference, value)
            }

            fn remove(
                &self,
                reference: &CredentialReferenceId,
            ) -> Result<bool, CredentialStoreError> {
                native_remove(reference)
            }
        }
    };
}

#[cfg(target_os = "linux")]
native_credential_store!(LinuxCredentialStore);

#[cfg(target_os = "windows")]
native_credential_store!(WindowsCredentialStore);

#[cfg(target_os = "macos")]
native_credential_store!(MacOsCredentialStore);

#[cfg(target_os = "linux")]
pub type PlatformCredentialStore = LinuxCredentialStore;

#[cfg(target_os = "windows")]
pub type PlatformCredentialStore = WindowsCredentialStore;

#[cfg(target_os = "macos")]
pub type PlatformCredentialStore = MacOsCredentialStore;
