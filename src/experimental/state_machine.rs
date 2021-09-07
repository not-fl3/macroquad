use crate::experimental::coroutines::Coroutine;

use crate::experimental::scene;

type UpdateFn<T> = Box<dyn FnMut(&mut scene::RefMut<T>, f32)>;
type CoroutineFn<T> = Box<dyn FnMut(&mut scene::RefMut<T>) -> Coroutine>;
type OnEndFn<T> = Box<dyn FnMut(&mut scene::RefMut<T>)>;

pub struct State<T: 'static> {
    update: Option<UpdateFn<T>>,
    coroutine: Option<CoroutineFn<T>>,
    on_end: Option<OnEndFn<T>>,
}

impl<T> State<T> {
    pub fn new() -> Self {
        State {
            update: None,
            coroutine: None,
            on_end: None,
        }
    }

    pub fn update(self, update: impl FnMut(&mut scene::RefMut<T>, f32) + 'static) -> Self {
        State {
            update: Some(Box::new(update)),
            ..self
        }
    }

    pub fn coroutine(
        self,
        coroutine: impl FnMut(&mut scene::RefMut<T>) -> Coroutine + 'static,
    ) -> Self {
        State {
            coroutine: Some(Box::new(coroutine)),
            ..self
        }
    }

    pub fn on_end(self, on_end: impl FnMut(&mut scene::RefMut<T>) + 'static) -> Self {
        State {
            on_end: Some(Box::new(on_end)),
            ..self
        }
    }
}

pub enum StateMachine<T: 'static> {
    Ready(StateMachineOwned<T>),
    InUse {
        next_state: Option<usize>,
        current_state: usize,
    },
}

impl<T: scene::Node + 'static> StateMachine<T> {
    pub fn new() -> StateMachine<T> {
        StateMachine::Ready(StateMachineOwned::new())
    }

    pub fn add_state(&mut self, id: usize, state: State<T>) {
        match self {
            StateMachine::Ready(state_machine) => state_machine.insert(id, state),
            _ => panic!(),
        }
    }

    pub fn take(&mut self) -> StateMachineOwned<T> {
        let current_state = self.state();
        match std::mem::replace(
            self,
            StateMachine::InUse {
                next_state: None,
                current_state,
            },
        ) {
            StateMachine::InUse { .. } => panic!(),
            StateMachine::Ready(state_machine) => state_machine,
        }
    }

    fn put_back(&mut self, mut state_machine: StateMachineOwned<T>) {
        match self {
            StateMachine::Ready(_) => panic!(),
            StateMachine::InUse { next_state, .. } => {
                if let Some(next_state) = next_state {
                    state_machine.set_state(*next_state);
                }
            }
        }
        *self = StateMachine::Ready(state_machine);
    }

    pub fn set_state(&mut self, state: usize) {
        match self {
            StateMachine::Ready(state_machine) => {
                state_machine.set_state(state);
            }
            StateMachine::InUse {
                ref mut next_state, ..
            } => {
                *next_state = Some(state);
            }
        }
    }

    pub fn state(&self) -> usize {
        match self {
            StateMachine::Ready(state_machine) => state_machine.state(),
            StateMachine::InUse {
                ref current_state, ..
            } => *current_state,
        }
    }

    /// A hack to update a state machine being part of an updating struct
    pub fn update_detached<'a, F>(mut t: scene::RefMut<T>, mut f: F)
    where
        F: FnMut(&mut scene::RefMut<T>) -> &mut StateMachine<T>,
    {
        let mut state_machine = f(&mut t).take();
        state_machine.update(&mut t, crate::time::get_frame_time());

        // coroutine may want to access this node by its handle
        // but while we have borrowed t to RefMut - it will fail this attempt
        // so here we drop t, poll the coroutine and than getting t back by its handle
        let handle = t.handle();
        drop(t);
        if let Some(ref mut coroutine) = state_machine.active_coroutine {
            coroutine.poll(crate::time::get_frame_time() as _);
        }
        let mut t = crate::experimental::scene::get_node(handle);

        f(&mut t).put_back(state_machine);
    }

    pub fn update<'a>(&mut self, t: &'a mut scene::RefMut<T>) {
        match self {
            StateMachine::Ready(state_machine) => {
                state_machine.update(t, crate::time::get_frame_time())
            }
            _ => panic!(),
        }
    }
}

pub struct StateMachineOwned<T: 'static> {
    states: Vec<State<T>>,
    active_coroutine: Option<Coroutine>,
    next_state: Option<usize>,
    current_state: usize,
}

impl<T: 'static> StateMachineOwned<T> {
    const MAX_STATE: usize = 32;

    pub fn new() -> Self {
        let mut states = vec![];
        for _ in 0..Self::MAX_STATE {
            states.push(State::new());
        }
        StateMachineOwned {
            states,
            active_coroutine: None,
            next_state: None,
            current_state: 0,
        }
    }

    pub fn insert(&mut self, id: usize, state: State<T>) {
        assert!(id < Self::MAX_STATE);

        self.states[id] = state;
    }

    pub fn set_state(&mut self, state: usize) {
        self.next_state = Some(state);
    }

    pub fn state(&self) -> usize {
        self.current_state
    }

    fn update(&mut self, player: &mut scene::RefMut<T>, dt: f32) {
        if let Some(next_state) = self.next_state {
            if next_state != self.current_state {
                if let Some(on_end) = &mut self.states[self.current_state].on_end {
                    on_end(player);
                }
                if let Some(coroutine) = &mut self.states[next_state].coroutine {
                    let mut coroutine = coroutine(player);
                    coroutine.set_manual_poll();
                    self.active_coroutine = Some(coroutine);
                }
            }
            self.current_state = next_state;
            self.next_state = None;
        }

        if let Some(update) = self.states[self.current_state].update.as_mut() {
            (update)(player, dt);
        }
    }
}
