use std::{any::Any, marker::PhantomData, ops::Drop};

#[rustfmt::skip]
pub trait Node {
    fn ready(_node: RefMut<Self>) where Self: Sized {}
    fn update(_node: RefMut<Self>) where Self: Sized  {}
    fn draw(_node: RefMut<Self>) where Self: Sized  {}
}

trait NodeTyped<T> {
    fn self_node(&self) -> &T;
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

#[derive(Clone, Copy, Debug, PartialEq)]
struct Id {
    id: usize,
    generation: u64,
}

pub struct Handle<T: 'static> {
    id: Option<Id>,
    _marker: PhantomData<T>,
}

unsafe impl<T: 'static> Send for Handle<T> {}

#[derive(Clone, Copy, Debug)]
pub(crate) struct HandleUntyped(Id);

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

pub(crate) struct Lens<T> {
    handle: HandleUntyped,
    offset: isize,
    _marker: PhantomData<T>,
}

impl<T> Lens<T> {
    pub fn get(&mut self) -> Option<&mut T> {
        let node = get_untyped_node(self.handle)?;

        Some(unsafe { &mut *((node.data as *mut u8).offset(self.offset) as *mut T) })
    }
}

impl<T> Handle<T> {
    pub(crate) fn lens<F, T1>(&self, f: F) -> Lens<T1>
    where
        F: for<'r> FnOnce(&'r mut T) -> &'r mut T1,
    {
        assert!(self.id.is_some());

        let offset = unsafe {
            let mut base = std::mem::MaybeUninit::<T>::uninit();
            let field = f(std::mem::transmute(base.as_mut_ptr())) as *mut _ as *mut u8;

            (field as *mut u8).offset_from(base.as_mut_ptr() as *mut u8)
        };

        Lens {
            handle: HandleUntyped(self.id.unwrap()),
            offset,
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

    pub fn persist(&self) {
        unsafe { get_scene() }.nodes[self.handle.id.unwrap().id]
            .as_mut()
            .unwrap()
            .permanent = true;
    }

    pub fn delete(self) {
        assert!(self.handle.id.is_some());

        unsafe {
            *self.used = false;
        }
        unsafe { get_scene() }.delete(self.handle.id.unwrap());
        std::mem::forget(self);
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
    used: *mut bool,
    vtable: *mut (),
    handle: HandleUntyped,

    _marker: PhantomData<&'a ()>,
}

impl<'a> RefMutAny<'a> {
    pub fn handle<T>(&self) -> Handle<T> {
        Handle {
            id: Some(self.handle.0),
            _marker: PhantomData,
        }
    }

    pub fn delete(self) {
        unsafe {
            *self.used = false;
        }
        unsafe { get_scene() }.delete(self.handle.0);
        std::mem::forget(self);
    }

    fn to_typed<T>(self) -> RefMut<T> {
        let res = RefMut {
            data: self.data as *mut T,
            handle: Handle {
                id: Some(self.handle.0),
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
    id: Id,
    data: *mut (),
    vtable: *mut (),
    ready: *const fn(RefMut<()>),
    update: *const fn(RefMut<()>),
    draw: *const fn(RefMut<()>),
    data_len: usize,
    permanent: bool,
    initialized: bool,
    used: *mut bool,
}
unsafe impl Sync for Scene {}

impl Cell {
    fn new<T: Node + 'static>(id: Id, data: *mut (), vtable: *mut (), used: *mut bool) -> Self {
        Cell {
            id,
            data,
            vtable,
            used,
            permanent: false,
            ready: unsafe {
                std::mem::transmute(&(Node::ready as fn(RefMut<T>)) as *const fn(RefMut<T>))
            },
            update: unsafe {
                std::mem::transmute(&(Node::update as fn(RefMut<T>)) as *const fn(RefMut<T>))
            },
            draw: unsafe {
                std::mem::transmute(&(Node::draw as fn(RefMut<T>)) as *const fn(RefMut<T>))
            },
            data_len: std::mem::size_of::<T>(),
            initialized: false,
        }
    }

    fn update<T: Node + 'static>(&mut self, data: T) {
        assert!(std::mem::size_of::<T>() <= self.data_len);

        let trait_obj = &data as &dyn NodeAny;
        let (_, vtable) = unsafe { std::mem::transmute::<_, (*mut (), *mut ())>(trait_obj) };

        self.vtable = vtable;
        self.ready =
            unsafe { std::mem::transmute(&(Node::ready as fn(RefMut<T>)) as *const fn(RefMut<T>)) };
        self.update = unsafe {
            std::mem::transmute(&(Node::update as fn(RefMut<T>)) as *const fn(RefMut<T>))
        };
        self.draw =
            unsafe { std::mem::transmute(&(Node::draw as fn(RefMut<T>)) as *const fn(RefMut<T>)) };

        unsafe {
            std::ptr::copy_nonoverlapping::<T>(&data as *const _ as *mut _, self.data as *mut _, 1);
        }
        self.id.generation += 1;
        self.initialized = false;
        self.permanent = false;

        std::mem::forget(data);
    }
}

struct Scene {
    dense: Vec<Id>,
    dense_ongoing: Vec<Result<Id, Id>>,
    nodes: Vec<Option<Cell>>,
    arena: bumpalo::Bump,
    camera: Option<Box<dyn crate::camera::Camera>>,

    free_nodes: Vec<Cell>,
}

impl Scene {
    pub fn new() -> Self {
        Scene {
            dense: vec![],
            dense_ongoing: vec![],
            nodes: Vec::new(),
            arena: bumpalo::Bump::new(),
            free_nodes: Vec::new(),
            camera: None,
        }
    }

    pub fn clear(&mut self) {
        for cell in &mut self.nodes {
            if let Some(Cell {
                permanent: false, ..
            }) = cell
            {
                if let Some(cell) = cell.take() {
                    assert!(unsafe { *cell.used == false });

                    let ix = self.dense.iter().position(|i| *i == cell.id).unwrap();
                    self.dense.remove(ix);

                    self.free_nodes.push(cell);
                }
            }
        }
    }

    pub fn get_any(&self, handle: HandleUntyped) -> Option<RefMutAny> {
        let handle = handle.0;
        let cell = self.nodes.get(handle.id)?;

        if cell.is_none() {
            return None;
        }
        let cell = cell.as_ref().unwrap();

        if cell.id.generation != handle.generation {
            return None;
        }

        if unsafe { *cell.used } {
            return None;
        }

        unsafe { *cell.used = true };

        Some(RefMutAny {
            data: cell.data,
            vtable: cell.vtable,
            handle: HandleUntyped(cell.id),
            used: cell.used,

            _marker: PhantomData,
        })
    }

    pub fn get<T>(&self, handle: Handle<T>) -> Option<RefMut<T>> {
        if handle.id.is_none() {
            return None;
        }
        let ref_mut_any = self.get_any(HandleUntyped(handle.id.unwrap()))?;
        Some(ref_mut_any.to_typed())
    }

    fn iter(&self) -> MagicVecIterator {
        MagicVecIterator {
            n: 0,
            len: self.dense.len(),
        }
    }

    fn add_node<T: Node + 'static>(&mut self, data: T) -> Handle<T> {
        let id;

        if let Some(i) = self
            .free_nodes
            .iter()
            .position(|free_node| free_node.data_len >= std::mem::size_of::<T>())
        {
            let mut free_node = self.free_nodes.remove(i);

            free_node.update::<T>(data);

            id = free_node.id;

            self.nodes[id.id] = Some(free_node);
        } else {
            let trait_obj = &data as &dyn NodeAny;
            let (_, vtable) = unsafe { std::mem::transmute::<_, (*mut (), *mut ())>(trait_obj) };

            let data = self.arena.alloc(data) as *mut _ as *mut _;
            let used = self.arena.alloc(false) as *mut _ as *mut _;
            id = Id {
                id: self.nodes.len(),
                generation: 0,
            };
            self.nodes
                .push(Some(Cell::new::<T>(id, data, vtable, used)));
        }

        self.dense.push(id);

        Handle {
            id: Some(id),
            _marker: PhantomData,
        }
    }

    pub fn delete(&mut self, id: Id) {
        if let Some(node) = self.nodes[id.id].take() {
            assert_eq!(node.id.generation, id.generation);

            self.dense_ongoing.push(Err(id));

            self.free_nodes.push(node);
        }
    }

    pub fn update(&mut self) {
        for node in &mut self.iter() {
            let cell = self.nodes[node.handle.0.id].as_mut().unwrap();
            if cell.initialized == false {
                cell.initialized = true;

                let node: RefMut<()> = node.to_typed::<()>();
                unsafe { (*cell.ready)(node) };
            }
        }

        for node in &mut self.iter() {
            let cell = self.nodes[node.handle.0.id].as_mut().unwrap();
            let node: RefMut<()> = node.to_typed::<()>();
            unsafe { (*cell.update)(node) };
        }

        if let Some(ref camera) = self.camera {
            crate::prelude::set_camera(&**camera);
        }
        for node in &mut self.iter() {
            let cell = self.nodes[node.handle.0.id].as_mut().unwrap();
            let node: RefMut<()> = node.to_typed::<()>();
            unsafe { (*cell.draw)(node) };
        }
        if self.camera.is_some() {
            crate::prelude::set_default_camera();
        }

        for id in self.dense_ongoing.drain(0..) {
            match id {
                Ok(id) => {
                    self.dense.push(id);
                }
                Err(id) => {
                    let ix = self.dense.iter().position(|i| *i == id).unwrap();
                    self.dense.remove(ix);
                }
            }
        }
    }
}

pub struct MagicVecIterator {
    n: usize,
    len: usize,
}

impl Iterator for MagicVecIterator {
    type Item = RefMutAny<'static>;

    fn next(&mut self) -> Option<RefMutAny<'static>> {
        let scene = unsafe { get_scene() };
        let nodes = &mut scene.nodes;
        let dense = &scene.dense;
        if self.n >= self.len {
            return None;
        }
        let ix = dense[self.n];
        let cell = &nodes[ix.id];
        self.n += 1;

        if cell.is_none() {
            return self.next();
        }
        let cell = cell.as_ref().unwrap();

        if unsafe { *cell.used } {
            return self.next();
        }

        unsafe { *cell.used = true };

        Some(RefMutAny {
            data: cell.data,
            vtable: cell.vtable,
            handle: HandleUntyped(cell.id),
            used: cell.used,
            _marker: PhantomData,
        })
    }
}

static mut SCENE: Option<Scene> = None;

unsafe fn get_scene() -> &'static mut Scene {
    SCENE.get_or_insert(Scene::new())
}

pub(crate) fn allocated_memory() -> usize {
    unsafe { get_scene() }.arena.allocated_bytes()
}

pub fn clear() {
    crate::experimental::coroutines::stop_all_coroutines();

    unsafe { get_scene() }.clear()
}

/// Get node and panic if the node is borrowed or deleted
pub fn get_node<T: Node>(handle: Handle<T>) -> RefMut<T> {
    unsafe { get_scene() }.get(handle).unwrap()
}

pub fn try_get_node<T: Node>(handle: Handle<T>) -> Option<RefMut<T>> {
    unsafe { get_scene() }.get(handle)
}

pub(crate) fn get_untyped_node(handle: HandleUntyped) -> Option<RefMutAny<'static>> {
    unsafe { get_scene() }.get_any(handle)
}

pub fn set_camera(camera: impl crate::camera::Camera + Clone + 'static) {
    unsafe { get_scene() }.camera = Some(Box::new(camera.clone()));
}

pub fn add_node<T: Node>(node: T) -> Handle<T> {
    unsafe { get_scene() }.add_node(node)
}

pub(crate) fn update() {
    unsafe { get_scene() }.update()
}

pub fn all_nodes() -> MagicVecIterator {
    unsafe { get_scene() }.iter()
}

pub fn find_node_by_type<T: Any>() -> Option<RefMut<T>> {
    unsafe { get_scene() }
        .iter()
        .find(|node| node.is::<T>())
        .map(|node| node.to_typed())
}

pub fn find_nodes_by_type<T: Any>() -> impl Iterator<Item = RefMut<T>> {
    unsafe { get_scene() }
        .iter()
        .filter(|node| node.is::<T>())
        .map(|node| node.to_typed())
}
