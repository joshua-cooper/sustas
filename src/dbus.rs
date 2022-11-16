//! Utilities for working with dbus.

use std::cmp::PartialEq;
use zbus::{
    fdo::ObjectManagerProxy,
    zvariant::{OwnedObjectPath, Value},
};

/// Finds the path of an interface with a given property value.
///
/// # Errors
///
/// Returns an error when communication with dbus fails.
pub async fn find_path<V>(
    object_manager: &ObjectManagerProxy<'_>,
    interface: &str,
    property_name: &str,
    property_value: &V,
) -> zbus::Result<Option<OwnedObjectPath>>
where
    V: PartialEq + Send + Sync + ?Sized,
    for<'a> &'a V: TryFrom<&'a Value<'a>>,
{
    for (path, interfaces) in object_manager.get_managed_objects().await? {
        if let Some(properties) = interfaces.get(interface) {
            if properties
                .get(property_name)
                .and_then(|p| p.downcast_ref::<V>())
                == Some(property_value)
            {
                return Ok(Some(path));
            }
        }
    }

    Ok(None)
}
