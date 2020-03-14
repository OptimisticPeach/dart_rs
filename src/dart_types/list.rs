use std::marker::PhantomData;
use crate::dart_handle::{UnverifiedDartHandle, DartHandle, Error};
use crate::dart_types::DartType;
use std::thread::LocalKey;
use std::ops::{RangeBounds, Deref, Index, IndexMut};
use std::cell::UnsafeCell;
use crate::dart_unwrap;
use crate::dart_types::integer::Integer;

#[derive(Copy, Clone)]
pub struct List<T> {
    _phantom: PhantomData<*mut T>,
    handle: UnverifiedDartHandle
}

impl<T: DartType> List<T> {
    pub fn new(length: usize) -> Self {
        let handle = T::THIS.with(|x| x.new_list_of_self_as_type(length));
        Self {
            handle: dart_unwrap!(handle),
            _phantom: PhantomData
        }
    }
}

impl List<UnverifiedDartHandle> {
    pub fn new_dynamic(length: usize) -> Self {
        let handle = UnverifiedDartHandle::new_list(length);
        Self {
            handle: dart_unwrap!(handle),
            _phantom: PhantomData
        }
    }
}

impl<T> List<T> {
    pub fn length(&self) -> usize {
        dart_unwrap!(self.handle.list_length())
    }

    pub fn get_range(&self, range: impl std::ops::RangeBounds<usize>) -> Result<Self, Error> {
        self
            .handle
            .list_get_range(range)
            .map(
                |handle| Self { handle, _phantom: PhantomData }
            )
    }

    pub fn iterator(&self) -> Result<UnverifiedDartHandle, Error> {
        self
            .handle
            .invoke(UnverifiedDartHandle::string_from_str("iterator"), &mut [])
    }

    pub fn reversed(&self) -> Result<UnverifiedDartHandle, Error> {
        self
            .handle
            .invoke(UnverifiedDartHandle::string_from_str("reversed"), &mut [])
    }
}

unsafe impl<T: 'static> DartHandle for List<T> {
    fn handle(&self) -> dart_sys::Dart_Handle {
        self.handle.handle()
    }
    fn safe_handle(&self) -> UnverifiedDartHandle {
        self.handle
    }
    fn from_handle(handle: UnverifiedDartHandle) -> Result<Self, UnverifiedDartHandle> {
        if handle.is_list() {
            Ok(
                Self {
                    handle,
                    _phantom: PhantomData
                }
            )
        } else {
            Err(handle)
        }
    }
}

impl<T> Deref for List<T> {
    type Target = UnverifiedDartHandle;
    fn deref(&self) -> &UnverifiedDartHandle {
        &self.handle
    }
}

impl<T: DartType> DartType for List<T> {
    const THIS: &'static LocalKey<UnverifiedDartHandle> = {
        thread_local! {
            #[allow(non_upper_case_globals)]
            pub static ListType: UnverifiedDartHandle = {
                let empty = UnverifiedDartHandle::new_list(0).ok().unwrap();
                empty.get_instance_type().ok().unwrap()
            };
        }
        &ListType
    };
}

impl<T: DartHandle> ListLike<T> for List<T> {
    fn get_first(&self) -> T {
        let handle = self.handle.invoke(UnverifiedDartHandle::string_from_str("first"), &mut []);
        let handle = dart_unwrap!(handle);
        T::from_handle(handle).ok().unwrap()
    }

    fn get_last(&self) -> T {
        let handle = self.handle.invoke(UnverifiedDartHandle::string_from_str("last"), &mut []);
        let handle = dart_unwrap!(handle);
        T::from_handle(handle).ok().unwrap()
    }

    fn set_at(&mut self, idx: usize, item: T) {
        let handle = self.handle.op_idx_assign(*Integer::from(idx), item.safe_handle());
        dart_unwrap!(handle)
    }
    fn get_at(&self, idx: usize) -> T {
        let handle = self.handle.op_idx(*Integer::from(idx));
        let handle = dart_unwrap!(handle);
        T::from_handle(handle).ok().unwrap()
    }

    fn len(&self) -> usize {
        self.length()
    }
}

pub trait ListLike<T> {
    fn get_first(&self) -> T;
    fn get_last(&self) -> T;

    fn set_at(&mut self, idx: usize, item: T);
    fn get_at(&self, idx: usize) -> T;

    fn slice<Q: RangeBounds<usize>>(&self, slice: Q) -> ListView<'_, T, Self> {
        let start = slice.start_bound();
        let start = match start {
            std::ops::Bound::Unbounded => 0,
            std::ops::Bound::Included(x) => *x,
            std::ops::Bound::Excluded(x) => *x + 1
        };
        let end = slice.end_bound();
        let end = match end {
            std::ops::Bound::Unbounded => self.len() - 1,
            std::ops::Bound::Included(x) => *x,
            std::ops::Bound::Excluded(x) => *x - 1
        };
        let len = end - start;
        ListView::new(start, len, self)
    }
    fn slice_mut<Q: RangeBounds<usize>>(&mut self, slice: Q) -> ListViewMut<'_, T, Self> where T: Clone {
        let start = slice.start_bound();
        let start = match start {
            std::ops::Bound::Unbounded => 0,
            std::ops::Bound::Included(x) => *x,
            std::ops::Bound::Excluded(x) => *x + 1
        };
        let end = slice.end_bound();
        let end = match end {
            std::ops::Bound::Unbounded => self.len() - 1,
            std::ops::Bound::Included(x) => *x,
            std::ops::Bound::Excluded(x) => *x - 1
        };
        let len = end - start;
        ListViewMut::new(start, len, self)
    }

    fn len(&self) -> usize;
}

pub struct ListView<'a, T, L: ListLike<T> + ?Sized = List<T>> {
    list: &'a L,
    cached_items: Vec<UnsafeCell<Option<T>>>,
    start: usize
}

impl<'a, T, L: ListLike<T> + ?Sized> ListView<'a, T, L> {
    fn new(start: usize, len: usize, list: &'a L) -> Self {
        ListView {
            list,
            cached_items: (0..len).map(|_| UnsafeCell::new(None)).collect(),
            start,
        }
    }
}

impl<'a, T, L: ListLike<T> + ?Sized> Index<usize> for ListView<'a, T, L> {
    type Output = T;
    fn index(&self, idx: usize) -> &T {
        //SAFETY:
        // Make sure that the list's items are never changed after
        // handing out a reference to them. Since references are
        // only ever handed out when there is a `Some` variant in
        // self.cached_items[idx], there is always an item there,
        // and the values at those locations are only ever modified
        // when the spot is populated by a `None`.
        unsafe {
            let item = &self.cached_items[idx];
            let item = item.get();
            if (*item).is_none() {
                *item = Some(self.list.get_at(idx + self.start))
            }
            (*(item as *const Option<T>)).as_ref().unwrap()
        }
    }
}

enum Item<T> {
    Read(T),
    PossiblyModified(T),
    None,
}

impl<T> Item<T> {
    fn get_ref(&self) -> Option<&T> {
        match self {
            Item::Read(x) | Item::PossiblyModified(x) => Some(x),
            Item::None => None
        }
    }
    fn make_mut(&mut self) -> Option<&mut T> {
        match self {
            Item::Read(_) => {
                let value = std::mem::replace(self, Item::None);
                if let Item::Read(x) = value {
                    std::mem::replace(self, Item::PossiblyModified(x));
                    if let Item::PossiblyModified(x) = self {
                        Some(x)
                    } else {
                        unsafe { std::hint::unreachable_unchecked() }
                    }
                } else {
                    unsafe { std::hint::unreachable_unchecked() }
                }
            },
            Item::PossiblyModified(x) => Some(x),
            Item::None => None
        }
    }
    fn is_none(&self) -> bool {
        if let Item::None = self {
            true
        } else {
            false
        }
    }
}

pub struct ListViewMut<'a, T: Clone, L: ListLike<T> + ?Sized = List<T>> {
    list: &'a mut L,
    cached_items: Vec<UnsafeCell<Item<T>>>,
    start: usize
}

impl<'a, T: Clone, L: ListLike<T> + ?Sized> ListLike<T> for ListViewMut<'a, T, L> {
    fn get_first(&self) -> T {
        self[0].clone()
    }
    fn get_last(&self) -> T {
        self[self.len() - 1].clone()
    }

    fn set_at(&mut self, idx: usize, item: T) {
        self[idx] = item;
    }
    fn get_at(&self, idx: usize) -> T {
        self[idx].clone()
    }

    fn len(&self) -> usize {
        self.cached_items.len()
    }
}

impl<'a, T: Clone, L: ListLike<T> + ?Sized> ListViewMut<'a, T, L> {
    fn new(start: usize, len: usize, list: &'a mut L) -> Self {
        ListViewMut {
            list,
            cached_items: (0..len).map(|_| UnsafeCell::new(Item::None)).collect(),
            start,
        }
    }
}

impl<'a, T: Clone, L: ListLike<T> + ?Sized> Index<usize> for ListViewMut<'a, T, L> {
    type Output = T;
    fn index(&self, idx: usize) -> &T {
        //SAFETY:
        // Since rust prevents us from indexing mutably _and_
        // [im]mutably at the same time, we don't have to worry about
        // overwriting a preexisting entry.
        // For more, please see `<ListView<'a, T> as Index<usize>>::index`'s
        // unsafety note.
        unsafe {
            let item = &self.cached_items[idx];
            let item = item.get();
            if (*item).is_none() {
                *item = Item::Read(self.list.get_at(idx + self.start));
            }
            (*(item as *const Item<T>)).get_ref().unwrap()
        }
    }
}

impl<'a, T: Clone, L: ListLike<T> + ?Sized> IndexMut<usize> for ListViewMut<'a, T, L> {
    fn index_mut(&mut self, idx: usize) -> &mut T {
        //SAFETY:
        // Make sure that the list's item states are never changed after
        // handing out a mutable reference to them. Since the references
        // have a lifetime attached to them, it is effectively impossible
        // for this to be called in a potentially conflicting way.
        // For more: See `<ListView<'a, T> as Index<usize>>::index`'s
        // unsafety note.
        unsafe {
            let item = &self.cached_items[idx];
            let item = item.get();
            if (*item).is_none() {
                *item = Item::PossiblyModified(self.list.get_at(idx + self.start));
            }
            (*item).make_mut().unwrap()
        }
    }
}

impl<'a, T: Clone, L: ListLike<T> + ?Sized> Drop for ListViewMut<'a, T, L> {
    fn drop(&mut self) {
        for (idx, i) in self.cached_items.iter().enumerate() {
            let i = i.get();
            unsafe {
                let item = &*i;
                match item {
                    Item::None | Item::Read(_) => {},
                    Item::PossiblyModified(x) => self.list.set_at(idx + self.start, x.clone())
                }
            }
        }
    }
}
