use std::any::Any;
use std::any::TypeId;
use std::any::type_name;
use std::borrow::Cow;
use std::sync::Arc;
use std::fmt::Debug;
pub trait Resource: Any + Debug + Send + Sync + 'static {
    fn name(&self) -> Cow<str> {
        type_name::<Self>().into()
    }
    fn close(self: Arc<Self>) {}
}

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
}
