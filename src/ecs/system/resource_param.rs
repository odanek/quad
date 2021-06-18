use crate::ecs::resource::Resource;
use std::{fmt::Debug, ops::Deref};

pub struct Res<'w, T: Resource> {
    pub(crate) value: &'w T,
}

impl<'w, T: Resource> Debug for Res<'w, T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Res").field(&self.value).finish()
    }
}

impl<'w, T: Resource> Deref for Res<'w, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.value
    }
}

impl<'w, T: Resource> AsRef<T> for Res<'w, T> {
    #[inline]
    fn as_ref(&self) -> &T {
        self.deref()
    }
}

pub struct ResMut<'a, T: Resource> {
    pub(crate) value: &'a mut T,
}

// impl<$($generics),* $(: $traits)?> Deref for $name<$($generics),*> {
//     type Target = $target;

//     #[inline]
//     fn deref(&self) -> &Self::Target {
//         self.value
//     }
// }

// impl<$($generics),* $(: $traits)?> DerefMut for $name<$($generics),*> {
//     #[inline]
//     fn deref_mut(&mut self) -> &mut Self::Target {
//         self.set_changed();
//         self.value
//     }
// }

// impl<$($generics),* $(: $traits)?> AsRef<$target> for $name<$($generics),*> {
//     #[inline]
//     fn as_ref(&self) -> &$target {
//         self.deref()
//     }
// }

// impl<$($generics),* $(: $traits)?> AsMut<$target> for $name<$($generics),*> {
//     #[inline]
//     fn as_mut(&mut self) -> &mut $target {
//         self.deref_mut()
//     }
// }