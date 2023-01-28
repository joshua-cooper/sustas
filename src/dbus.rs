//! Utilities for working with dbus.

use futures_util::{future::OptionFuture, stream::Next, StreamExt};
use std::cmp::PartialEq;
use zbus::{
    fdo::ManagedObjects,
    zvariant::{ObjectPath, Value},
    PropertyStream,
};
use zbus::{fdo::ObjectManagerProxy, zvariant::OwnedObjectPath};

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

/// Gets the path of an interface with a given property value.
///
/// If no property with the given name and value exsts on the given interface, this returns `None`.
pub fn get_object_path<'a, V>(
    objects: &'a ManagedObjects,
    interface_name: &str,
    property_name: &str,
    property_value: &V,
) -> Option<ObjectPath<'a>>
where
    V: PartialEq + ?Sized,
    for<'v> &'v V: TryFrom<&'v Value<'v>>,
{
    for (path, interfaces) in objects {
        let is_match = interfaces
            .get(interface_name)
            .and_then(|properties| properties.get(property_name))
            .and_then(|name| name.downcast_ref::<V>())
            .map(|name| name == property_value)
            .unwrap_or_default();

        if is_match {
            return Some(path.as_ref());
        }
    }

    None
}

/// Returns an [`OptionFuture`] future for the next change in a [`PropertyStream`].
pub fn option_change<'a, 'b, T>(
    stream: Option<&'a mut PropertyStream<'b, T>>,
) -> OptionFuture<Next<'a, PropertyStream<'b, T>>>
where
    T: Unpin,
{
    stream.map(StreamExt::next).into()
}
