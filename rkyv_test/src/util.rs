#[cfg(feature = "wasm")]
wasm_bindgen_test::wasm_bindgen_test_configure!();

use rkyv::{
    archived_root, archived_unsized_root,
    ser::{serializers::BufferSerializer, Serializer},
    Aligned, Deserialize, Serialize, SerializeUnsized,
};
#[cfg(feature = "std")]
use rkyv::{
    de::{adapters::SharedDeserializerAdapter, deserializers::AllocDeserializer},
    ser::adapters::SharedSerializerAdapter,
};

pub const BUFFER_SIZE: usize = 256;

#[cfg(feature = "std")]
pub type DefaultSerializer =
    SharedSerializerAdapter<BufferSerializer<Aligned<[u8; BUFFER_SIZE]>>>;

#[cfg(feature = "std")]
pub fn make_default_serializer() -> DefaultSerializer {
    SharedSerializerAdapter::new(BufferSerializer::new(Aligned([0u8; BUFFER_SIZE])))
}

#[cfg(feature = "std")]
pub fn unwrap_default_serializer(s: DefaultSerializer) -> Aligned<[u8; BUFFER_SIZE]> {
    s.into_inner().into_inner()
}

#[cfg(feature = "std")]
pub type DefaultDeserializer = SharedDeserializerAdapter<AllocDeserializer>;

#[cfg(feature = "std")]
pub fn make_default_deserializer() -> DefaultDeserializer {
    SharedDeserializerAdapter::new(AllocDeserializer)
}

#[cfg(not(feature = "std"))]
pub type DefaultSerializer = BufferSerializer<Aligned<[u8; BUFFER_SIZE]>>;

#[cfg(not(feature = "std"))]
pub fn make_default_serializer() -> DefaultSerializer {
    BufferSerializer::new(Aligned([0u8; BUFFER_SIZE]))
}

#[cfg(not(feature = "std"))]
pub fn unwrap_default_serializer(s: DefaultSerializer) -> Aligned<[u8; BUFFER_SIZE]> {
    s.into_inner()
}

#[cfg(not(feature = "std"))]
pub struct DefaultDeserializer;

#[cfg(not(feature = "std"))]
impl rkyv::Fallible for DefaultDeserializer {
    type Error = ();
}

#[cfg(not(feature = "std"))]
pub fn make_default_deserializer() -> DefaultDeserializer {
    DefaultDeserializer
}

pub fn test_archive<T: Serialize<DefaultSerializer>>(value: &T)
where
    T: PartialEq,
    T::Archived: PartialEq<T> + Deserialize<T, DefaultDeserializer>,
{
    let mut serializer = make_default_serializer();
    serializer
        .serialize_value(value)
        .expect("failed to archive value");
    let len = serializer.pos();
    let buffer = unwrap_default_serializer(serializer);

    let archived_value = unsafe { archived_root::<T>(&buffer.as_ref()[0..len]) };
    assert!(archived_value == value);
    let mut deserializer = make_default_deserializer();
    assert!(&archived_value.deserialize(&mut deserializer).unwrap() == value);
}

pub fn test_archive_ref<T: SerializeUnsized<DefaultSerializer> + ?Sized>(value: &T)
where
    T::Archived: PartialEq<T>,
{
    let mut serializer = make_default_serializer();
    serializer
        .serialize_unsized_value(value)
        .expect("failed to archive ref");
    let len = serializer.pos();
    let buffer = unwrap_default_serializer(serializer);

    let archived_ref = unsafe { archived_unsized_root::<T>(&buffer.as_ref()[0..len]) };
    assert!(archived_ref == value);
}

#[cfg(feature = "std")]
pub fn test_archive_container<
    T: Serialize<DefaultSerializer, Archived = U> + core::ops::Deref<Target = TV>,
    TV: ?Sized,
    U: core::ops::Deref<Target = TU>,
    TU: PartialEq<TV> + ?Sized,
>(
    value: &T,
) {
    let mut serializer = make_default_serializer();
    serializer
        .serialize_value(value)
        .expect("failed to archive ref");
    let len = serializer.pos();
    let buffer = unwrap_default_serializer(serializer);

    let archived_ref = unsafe { archived_root::<T>(&buffer.as_ref()[0..len]) };
    assert!(archived_ref.deref() == value.deref());
}