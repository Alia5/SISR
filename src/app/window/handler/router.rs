use std::{collections::HashMap, mem::Discriminant, sync::Arc};

use winit::event_loop::ActiveEventLoop;

use crate::app::window::{event::WindowRunnerEvent, runner::WindowRunner};

pub trait EventHandler {
    fn handle_event(
        &self,
        runner: &mut WindowRunner,
        event_loop: &ActiveEventLoop,
        event: &WindowRunnerEvent,
    );

    fn listen_events(&self) -> Vec<Discriminant<WindowRunnerEvent>>;
}

pub struct EventRouter {
    event_handler_map: HashMap<Discriminant<WindowRunnerEvent>, Arc<dyn EventHandler>>,
}

impl EventRouter {
    pub fn new() -> Self {
        Self {
            event_handler_map: HashMap::new(),
        }
    }

    pub fn register_multiple(&mut self, handlers: &[Arc<dyn EventHandler>]) {
        for handler in handlers {
            self.register(handler.clone());
        }
    }

    pub fn register(&mut self, handler: Arc<dyn EventHandler>) {
        for listen_event in handler.listen_events() {
            self.event_handler_map.insert(listen_event, handler.clone());
        }
    }

    pub fn handler_for(&self, event: &WindowRunnerEvent) -> Option<Arc<dyn EventHandler>> {
        let discriminant = std::mem::discriminant(event);
        self.event_handler_map.get(&discriminant).cloned()
    }
}
