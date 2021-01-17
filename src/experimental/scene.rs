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

#[derive(Clone, Copy, Debug)]
struct Id {
    id: usize,
    generation: u64,
}

pub struct Handle<T: 'static> {
    id: Option<Id>,
    _marker: PhantomData<T>,
}

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
    vtable: *mut (),
    handle: HandleUntyped,
    used: *mut bool,

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

        std::mem::forget(data);
    }
}

struct Scene {
    nodes: Vec<Option<Cell>>,
    arena: bumpalo::Bump,

    free_nodes: Vec<Cell>,
}

impl Scene {
    pub fn new() -> Self {
        Scene {
            nodes: Vec::new(),
            arena: bumpalo::Bump::new(),
            free_nodes: Vec::new(),
        }
    }

    pub fn clear(&mut self) {
        for cell in &self.nodes {
            if let Some(cell) = cell {
                assert!(unsafe { *cell.used == false },);
            }
        }

        self.nodes.clear()
    }

    pub fn get<T>(&self, handle: Handle<T>) -> Option<RefMut<T>> {
        if handle.id.is_none() {
            return None;
        }
        let handle = handle.id.unwrap();
        let cell = &self.nodes[handle.id];

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

        Some(RefMut {
            data: unsafe { &mut *(cell.data as *mut T) },
            handle: Handle {
                id: Some(cell.id),
                _marker: PhantomData,
            },
            used: cell.used,
        })
    }

    fn iter(&self) -> MagicVecIterator {
        MagicVecIterator {
            n: 0,
            len: self.nodes.len(),
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

        Handle {
            id: Some(id),
            _marker: PhantomData,
        }
    }

    pub fn delete(&mut self, id: Id) {
        if let Some(node) = self.nodes[id.id].take() {
            assert_eq!(node.id.generation, id.generation);

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

        for node in &mut self.iter() {
            let cell = self.nodes[node.handle.0.id].as_mut().unwrap();
            let node: RefMut<()> = node.to_typed::<()>();
            unsafe { (*cell.draw)(node) };
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
        let nodes = &mut unsafe { get_scene() }.nodes;
        if self.n >= self.len {
            return None;
        }
        let cell = &nodes[self.n];
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
    unsafe { get_scene() }.clear()
}

pub fn get_node<T: Node>(handle: Handle<T>) -> Option<RefMut<T>> {
    unsafe { get_scene() }.get(handle)
}

pub fn add_node<T: Node>(node: T) -> Handle<T> {
    unsafe { get_scene() }.add_node(node)
}

pub fn update() {
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

