use std::any::Any;
use std::any::TypeId;
use std::sync::Arc;

pub trait Resource: Any + Send + Sync + 'static {}

impl dyn Resource {
    #[inline(always)]
    fn is<T: Resource>(&self) -> bool {
        self.type_id() == TypeId::of::<T>()
    }

    #[inline(always)]
    #[allow(clippy::needless_lifetimes)]
    pub fn downcast_arc<'a, T: Resource>(
        self: &'a Arc<Self>
    ) -> Option<&'a Arc<T>> {
        if self.is::<T>() {
            let ptr = self as *const Arc<_> as *const Arc<T>;
            // TODO(piscisaureus): safety comment
            #[allow(clippy::undocumented_unsafe_blocks)]
            Some(unsafe { &*ptr })
        } else {
            None
        }
    }
    #[inline(always)]
    #[allow(clippy::needless_lifetimes)]
    pub fn downcast<'a, T: Resource>(
        self: &'a Box<Self>
    ) -> Option<&'a Box<T>> {
        if self.is::<T>() {
            let ptr = self as *const Box<_> as *const Box<T>;
            // TODO(piscisaureus): safety comment
            #[allow(clippy::undocumented_unsafe_blocks)]
            Some(unsafe { &*ptr })
        } else {
            None
        }
    }
}
