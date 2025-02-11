//! [`Archive`] implementations for std types.

pub mod chd;
pub mod shared;
#[cfg(feature = "validation")]
pub mod validation;

use crate::{
    de::Deserializer, Archive, ArchivePointee, ArchiveUnsized, Archived, Deserialize,
    DeserializeUnsized, Fallible, MetadataResolver, RelPtr, Serialize, SerializeUnsized,
};
use core::{
    borrow::Borrow,
    cmp, fmt, hash,
    ops::{Deref, DerefMut, Index, IndexMut},
    pin::Pin,
};

/// An archived [`String`].
///
/// Uses a [`RelPtr`] to a `str` under the hood.
#[derive(Debug)]
#[repr(transparent)]
pub struct ArchivedString(RelPtr<str>);

impl ArchivedString {
    /// Extracts a string slice containing the entire `ArchivedString`.
    pub fn as_str(&self) -> &str {
        self.deref()
    }

    /// Converts an `ArchivedString` into a mutable string slice.
    pub fn as_mut_str(&mut self) -> &mut str {
        self.deref_mut()
    }

    /// Gets the value of this archived string as a pinned mutable reference.
    pub fn str_pin(self: Pin<&mut Self>) -> Pin<&mut str> {
        unsafe { self.map_unchecked_mut(|s| s.deref_mut()) }
    }
}

impl cmp::Eq for ArchivedString {}

impl hash::Hash for ArchivedString {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.as_str().hash(state)
    }
}

impl cmp::Ord for ArchivedString {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.as_str().cmp(other.as_str())
    }
}

impl cmp::PartialEq for ArchivedString {
    fn eq(&self, other: &Self) -> bool {
        self.as_str() == other.as_str()
    }
}

impl cmp::PartialOrd for ArchivedString {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        self.as_str().partial_cmp(other.as_str())
    }
}

impl AsRef<str> for ArchivedString {
    fn as_ref(&self) -> &str {
        self.deref()
    }
}

impl AsMut<str> for ArchivedString {
    fn as_mut(&mut self) -> &mut str {
        self.deref_mut()
    }
}

impl Deref for ArchivedString {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.0.as_ptr() }
    }
}

impl DerefMut for ArchivedString {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.0.as_mut_ptr() }
    }
}

impl Borrow<str> for ArchivedString {
    fn borrow(&self) -> &str {
        self.deref().borrow()
    }
}

impl PartialEq<&str> for ArchivedString {
    fn eq(&self, other: &&str) -> bool {
        PartialEq::eq(self.as_str(), *other)
    }
}

impl PartialEq<ArchivedString> for &str {
    fn eq(&self, other: &ArchivedString) -> bool {
        PartialEq::eq(other.as_str(), *self)
    }
}

impl PartialEq<String> for ArchivedString {
    fn eq(&self, other: &String) -> bool {
        PartialEq::eq(self.as_str(), other.as_str())
    }
}

impl PartialEq<ArchivedString> for String {
    fn eq(&self, other: &ArchivedString) -> bool {
        PartialEq::eq(other.as_str(), self.as_str())
    }
}

impl fmt::Display for ArchivedString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self.as_str(), f)
    }
}

/// The resolver for `String`.
pub struct StringResolver {
    pos: usize,
    metadata_resolver: MetadataResolver<str>,
}

impl Archive for String {
    type Archived = ArchivedString;
    type Resolver = StringResolver;

    fn resolve(&self, pos: usize, resolver: StringResolver) -> Self::Archived {
        #[allow(clippy::unit_arg)]
        unsafe {
            ArchivedString(self.as_str().resolve_unsized(
                pos,
                resolver.pos,
                resolver.metadata_resolver,
            ))
        }
    }
}

impl<S: Fallible + ?Sized> Serialize<S> for String
where
    str: SerializeUnsized<S>,
{
    fn serialize(&self, serializer: &mut S) -> Result<Self::Resolver, S::Error> {
        Ok(StringResolver {
            pos: self.as_str().serialize_unsized(serializer)?,
            metadata_resolver: self.as_str().serialize_metadata(serializer)?,
        })
    }
}

impl<D: Fallible + ?Sized> Deserialize<String, D> for Archived<String>
where
    str: DeserializeUnsized<str, D>,
{
    fn deserialize(&self, deserializer: &mut D) -> Result<String, D::Error> {
        unsafe {
            let data_address = self.as_str().deserialize_unsized(deserializer)?;
            let metadata = self.0.metadata().deserialize(deserializer)?;
            let ptr = ptr_meta::from_raw_parts_mut(data_address, metadata);
            Ok(Box::<str>::from_raw(ptr).into())
        }
    }
}

/// An archived [`Box`].
///
/// This is a thin wrapper around a [`RelPtr`] to the archived type.
#[repr(transparent)]
pub struct ArchivedBox<T: ArchivePointee + ?Sized>(RelPtr<T>);

impl<T: ArchivePointee + ?Sized> fmt::Debug for ArchivedBox<T>
where
    T::ArchivedMetadata: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("ArchivedBox").field(&self.0).finish()
    }
}

impl<T: ArchivePointee + ?Sized> ArchivedBox<T> {
    /// Gets the value of this archived box as a pinned mutable reference.
    pub fn get_pin(self: Pin<&mut Self>) -> Pin<&mut T> {
        unsafe { self.map_unchecked_mut(|s| s.deref_mut()) }
    }
}

impl<T: ArchivePointee + ?Sized> Deref for ArchivedBox<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.0.as_ptr() }
    }
}

impl<T: ArchivePointee + ?Sized> DerefMut for ArchivedBox<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.0.as_mut_ptr() }
    }
}

impl<T: ArchivePointee + PartialEq<U> + ?Sized, U: ?Sized> PartialEq<Box<U>> for ArchivedBox<T> {
    fn eq(&self, other: &Box<U>) -> bool {
        self.deref().eq(other.deref())
    }
}

/// The resolver for `Box`.
pub struct BoxResolver<T> {
    pos: usize,
    metadata_resolver: T,
}

impl<T: ArchiveUnsized + ?Sized> Archive for Box<T> {
    type Archived = ArchivedBox<T::Archived>;
    type Resolver = BoxResolver<T::MetadataResolver>;

    fn resolve(&self, pos: usize, resolver: Self::Resolver) -> Self::Archived {
        unsafe {
            ArchivedBox(self.as_ref().resolve_unsized(
                pos,
                resolver.pos,
                resolver.metadata_resolver,
            ))
        }
    }
}

impl<T: SerializeUnsized<S> + ?Sized, S: Fallible + ?Sized> Serialize<S> for Box<T> {
    fn serialize(&self, serializer: &mut S) -> Result<Self::Resolver, S::Error> {
        Ok(BoxResolver {
            pos: self.as_ref().serialize_unsized(serializer)?,
            metadata_resolver: self.as_ref().serialize_metadata(serializer)?,
        })
    }
}

impl<T: ArchiveUnsized + ?Sized, D: Deserializer + ?Sized> Deserialize<Box<T>, D>
    for Archived<Box<T>>
where
    T::Archived: DeserializeUnsized<T, D>,
{
    fn deserialize(&self, deserializer: &mut D) -> Result<Box<T>, D::Error> {
        unsafe {
            let data_address = self.deref().deserialize_unsized(deserializer)?;
            let metadata = self.deref().deserialize_metadata(deserializer)?;
            let ptr = ptr_meta::from_raw_parts_mut(data_address, metadata);
            Ok(Box::from_raw(ptr))
        }
    }
}

/// An archived [`Vec`].
///
/// Uses a [`RelPtr`] to a `T` slice under the hood.
#[derive(Debug)]
#[repr(transparent)]
pub struct ArchivedVec<T>(RelPtr<[T]>);

impl<T> ArchivedVec<T> {
    /// Gets the elements of the archived vec as a slice.
    pub fn as_slice(&self) -> &[T] {
        self.deref()
    }

    /// Gets the elements of the archived vec as a mutable slice.
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        self.deref_mut()
    }

    /// Gets the element at the given index ot this archived vec as a pinned
    /// mutable reference.
    pub fn index_pin<I>(self: Pin<&mut Self>, index: I) -> Pin<&mut <[T] as Index<I>>::Output>
    where
        [T]: IndexMut<I>,
    {
        unsafe { self.map_unchecked_mut(|s| &mut s.deref_mut()[index]) }
    }
}

impl<T> Deref for ArchivedVec<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.0.as_ptr() }
    }
}

impl<T> DerefMut for ArchivedVec<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.0.as_mut_ptr() }
    }
}

/// The resolver for `Vec`.
pub struct VecResolver<T> {
    pos: usize,
    metadata_resolver: T,
}

impl<T: Archive> Archive for Vec<T> {
    type Archived = ArchivedVec<T::Archived>;
    type Resolver = VecResolver<MetadataResolver<[T]>>;

    fn resolve(&self, pos: usize, resolver: Self::Resolver) -> Self::Archived {
        #[allow(clippy::unit_arg)]
        unsafe {
            ArchivedVec(self.as_slice().resolve_unsized(
                pos,
                resolver.pos,
                resolver.metadata_resolver,
            ))
        }
    }
}

impl<T: Serialize<S>, S: Fallible + ?Sized> Serialize<S> for Vec<T>
where
    [T]: SerializeUnsized<S>,
{
    fn serialize(&self, serializer: &mut S) -> Result<Self::Resolver, S::Error> {
        Ok(VecResolver {
            pos: self.as_slice().serialize_unsized(serializer)?,
            metadata_resolver: self.as_slice().serialize_metadata(serializer)?,
        })
    }
}

impl<T: Archive, D: Fallible + ?Sized> Deserialize<Vec<T>, D> for Archived<Vec<T>>
where
    [T::Archived]: DeserializeUnsized<[T], D>,
{
    fn deserialize(&self, deserializer: &mut D) -> Result<Vec<T>, D::Error> {
        unsafe {
            let data_address = self.deref().deserialize_unsized(deserializer)?;
            let metadata = self.deref().deserialize_metadata(deserializer)?;
            let ptr = ptr_meta::from_raw_parts_mut(data_address, metadata);
            Ok(Box::<[T]>::from_raw(ptr).into())
        }
    }
}

impl<T: PartialEq<U>, U> PartialEq<Vec<U>> for ArchivedVec<T> {
    fn eq(&self, other: &Vec<U>) -> bool {
        self.as_slice().eq(other.as_slice())
    }
}
