use std::{any::Any, cell::UnsafeCell, marker::PhantomData, ops::Drop};

pub trait Node {
    fn update(&mut self) {}
    fn draw(&mut self) {}
}

trait NodeAny: Any + Node {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl<T: Node + 'static> NodeAny for T {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

pub struct Handle<T: 'static> {
    id: Option<usize>,
    _marker: PhantomData<T>,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct HandleUntyped(usize);

impl HandleUntyped {
    pub fn to_typed<T: 'static>(&self) -> Handle<T> {
        Handle {
            id: Some(self.0),
            _marker: PhantomData,
        }
    }
}

impl<T: 'static> std::fmt::Debug for Handle<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self.id)?;

        Ok(())
    }
}

impl<T: 'static> Clone for Handle<T> {
    fn clone(&self) -> Handle<T> {
        Handle {
            id: self.id,
            _marker: PhantomData,
        }
    }
}

impl<T: 'static> Copy for Handle<T> {}

impl<T: 'static> Handle<T> {
    pub fn null() -> Handle<T> {
        Handle {
            id: None,
            _marker: PhantomData,
        }
    }
}

pub struct RefMut<T: 'static> {
    data: *mut T,
    handle: Handle<T>,
    used: *mut bool,
}

impl<T: 'static> RefMut<T> {
    pub fn handle(&self) -> Handle<T> {
        Handle {
            id: self.handle.id,
            _marker: PhantomData,
        }
    }
}
impl<T> std::ops::Deref for RefMut<T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self.data }
    }
}

impl<T> std::ops::DerefMut for RefMut<T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.data }
    }
}

impl<T: 'static> Drop for RefMut<T> {
    fn drop(&mut self) {
        assert_eq!(unsafe { *self.used }, true);
        unsafe {
            *self.used = false;
        }
    }
}

pub struct RefMutAny<'a> {
    data: *mut (),
    vtable: *mut (),
    handle: usize,
    used: *mut bool,
    _marker: PhantomData<&'a ()>,
}

impl<'a> RefMutAny<'a> {
    pub fn handle<T>(&self) -> Handle<T> {
        Handle {
            id: Some(self.handle),
            _marker: PhantomData,
        }
    }

    fn into_ref_mut<T>(self) -> RefMut<T> {
        let res = RefMut {
            data: self.data as *mut T,
            handle: Handle {
                id: Some(self.handle),
                _marker: PhantomData::<T>,
            },
            used: self.used,
        };

        // "used" is now moved to RefMut and will be invalidated by RefMut's drop
        // no need for RefMutAny's drop than
        std::mem::forget(self);

        res
    }
}

impl<'a> std::ops::Deref for RefMutAny<'a> {
    type Target = dyn Any;

    fn deref(&self) -> &Self::Target {
        let trait_obj: &dyn NodeAny = unsafe { std::mem::transmute((self.data, self.vtable)) };

        trait_obj.as_any()
    }
}

impl<'a> std::ops::DerefMut for RefMutAny<'a> {
    fn deref_mut(&mut self) -> &mut dyn Any {
        let trait_obj: &mut dyn NodeAny = unsafe { std::mem::transmute((self.data, self.vtable)) };

        trait_obj.as_any_mut()
    }
}

impl<'a> Drop for RefMutAny<'a> {
    fn drop(&mut self) {
        assert_eq!(unsafe { *self.used }, true);
        unsafe {
            *self.used = false;
        }
    }
}

struct Cell {
    id: usize,
    data: *mut (),
    vtable: *mut (),
    used: UnsafeCell<bool>,
}
unsafe impl Sync for Scene {}

impl Cell {
    fn new(id: usize, data: *mut (), vtable: *mut ()) -> Self {
        Cell {
            id,
            data,
            vtable,
            used: UnsafeCell::new(false),
        }
    }

    fn as_trait_obj_mut(&mut self) -> &mut dyn NodeAny {
        unsafe { std::mem::transmute((self.data, self.vtable)) }
    }
}

struct Scene {
    nodes: Vec<Cell>,
    arena: bumpalo::Bump,

    // right now used only for coroutines
    active_node: Option<HandleUntyped>,
}

impl Scene {
    pub fn new() -> Self {
        Scene {
            nodes: Vec::new(),
            arena: bumpalo::Bump::new(),
            active_node: None,
        }
    }

    pub fn clear(&mut self) {
        for cell in &self.nodes {
            assert!(unsafe { *cell.used.get() == false },);
        }

        self.nodes.clear()
    }

    pub fn get<T>(&self, handle: Handle<T>) -> Option<RefMut<T>> {
        if handle.id.is_none() {
            return None;
        }
        let handle = handle.id.unwrap();
        let cell = &self.nodes[handle];

        if unsafe { *cell.used.get() } {
            return None;
        }

        unsafe { *cell.used.get() = true };

        Some(RefMut {
            data: unsafe { &mut *(cell.data as *mut T) },
            handle: Handle {
                id: Some(cell.id),
                _marker: PhantomData,
            },
            used: cell.used.get(),
        })
    }

    pub fn iter(&self) -> MagicVecIterator<'static> {
        let iter = unsafe { std::mem::transmute(self.nodes.iter()) };
        MagicVecIterator { iter }
    }

    pub fn push<T: Node>(&mut self, data: T) -> Handle<T> {
        let trait_obj = &data as &dyn NodeAny;
        let (_, vtable) = unsafe { std::mem::transmute::<_, (*mut (), *mut ())>(trait_obj) };

        let data = self.arena.alloc(data) as *mut _ as *mut _;

        self.nodes.push(Cell::new(self.nodes.len(), data, vtable));

        Handle {
            id: Some(self.nodes.len() - 1),
            _marker: PhantomData,
        }
    }
}

pub struct MagicVecIterator<'a> {
    iter: std::slice::Iter<'a, Cell>,
}

impl<'a> Iterator for MagicVecIterator<'a> {
    type Item = RefMutAny<'a>;

    fn next(&mut self) -> Option<RefMutAny<'a>> {
        let cell = self.iter.next()?;

        if unsafe { *cell.used.get() } {
            return self.next();
        }

        unsafe { *cell.used.get() = true };

        Some(RefMutAny {
            data: cell.data,
            vtable: cell.vtable,
            handle: cell.id,
            used: cell.used.get(),
            _marker: PhantomData,
        })
    }
}

static mut SCENE: Option<Scene> = None;

unsafe fn get_scene() -> &'static mut Scene {
    SCENE.get_or_insert(Scene::new())
}

pub(crate) fn active_node() -> Option<HandleUntyped> {
    unsafe { get_scene() }.active_node
}

pub fn clear() {
    unsafe { get_scene() }.clear()
}

pub fn get_node<T: Node>(handle: Handle<T>) -> Option<RefMut<T>> {
    unsafe { get_scene() }.get(handle)
}

pub fn add_node<T: Node>(node: T) -> Handle<T> {
    unsafe { get_scene() }.push(node)
}

pub fn update() {
    for node in &mut unsafe { get_scene() }.nodes {
        unsafe { get_scene() }.active_node = Some(HandleUntyped(node.id));

        let trait_obj = node.as_trait_obj_mut();
        trait_obj.update();
        trait_obj.draw();
    }

    unsafe { get_scene() }.active_node = None;
}

pub fn all_nodes() -> MagicVecIterator<'static> {
    unsafe { get_scene() }.iter()
}

pub fn find_node_by_type<T: Any>() -> Option<RefMut<T>> {
    unsafe { get_scene() }
        .iter()
        .find(|node| node.is::<T>())
        .map(|node| node.into_ref_mut())
}
