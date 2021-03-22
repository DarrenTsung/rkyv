//! [`Archive`] implementations for [`Uuid`](uuid::Uuid).

use crate::{Archive, ArchiveCopy, Deserialize, Fallible, Serialize};
use uuid::Uuid;

impl Archive for Uuid {
    type Archived = Uuid;
    type Resolver = ();

    fn resolve(&self, _: usize, _: Self::Resolver) -> Self::Archived {
        *self
    }
}

// Safety: UUID is Copy and doesn't need to transform its data during
// serialization
unsafe impl ArchiveCopy for Uuid {}

impl<S: Fallible + ?Sized> Serialize<S> for Uuid {
    fn serialize(&self, _: &mut S) -> Result<Self::Resolver, S::Error> {
        Ok(())
    }
}

impl<D: Fallible + ?Sized> Deserialize<Uuid, D> for Uuid {
    fn deserialize(&self, _: &mut D) -> Result<Uuid, D::Error> {
        Ok(*self)
    }
}
