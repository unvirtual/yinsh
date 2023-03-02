use std::collections::{HashMap};
use std::hash::Hash;

use crate::{core::game::UiAction};

use super::{
    element::Element,
    events::{Event, Message},
};

fn hashmap_vec_push<K, V>(hashmap: &mut HashMap<K, Vec<V>>, key: K, value: V)
where
    K: Eq + Hash + Clone,
{
    hashmap.entry(key.clone()).or_default();
    hashmap.get_mut(&key).unwrap().push(value);
}

pub type ElementId = usize;

pub struct Presenter {
    elements: HashMap<ElementId, Box<dyn Element>>,
    messages: HashMap<ElementId, Vec<Message>>,
    subscribers: HashMap<ElementId, Vec<ElementId>>,
    actions: Vec<UiAction>,
    events: Vec<Event>,
}

impl Presenter {
    pub fn new() -> Self {
        Self {
            elements: HashMap::new(),
            messages: HashMap::new(),
            subscribers: HashMap::new(),
            actions: vec![],
            events: vec![],
        }
    }

    pub fn add_element(&mut self, element: Box<dyn Element>) -> ElementId {
        let id = self.add_element_inactive(element);
        self.add_subscriber(id, id);
        id
    }

    pub fn add_element_inactive(&mut self, element: Box<dyn Element>) -> ElementId {
        let id = self.elements.len();
        self.elements.insert(id, element);
        id
    }

    pub fn get_actions(&self) -> Vec<UiAction> {
        self.actions.clone()
    }

    pub fn clear_all(&mut self) {
        self.elements.clear();
        self.messages.clear();
        self.subscribers.clear();
        self.actions.clear();
        self.events.clear();
    }

    pub fn schedule_event(&mut self, event: Event) {
        self.events.push(event);
    }

    pub fn handle_events(&mut self) {
        self.schedule_event(Event::Tick);

        // ensure that hovering works correctly, i.e. propagate MouseEnter/MouseLeave/MouseClicked events depending on z-value
        let mut mouse_hover_candidates = vec![];
        let mut mouse_clicked_candidates = vec![];

        for e in self.events.drain(0..) {
            self.elements.iter().for_each(|(id, element)| {
                element.handle_event(&e).into_iter().for_each(|msg| {
                    match msg {
                        Message::MouseInside | Message::MouseEntered | Message::MouseLeft => {
                            mouse_hover_candidates.push((*id, element.z_value(), msg))
                        }
                        Message::MouseClicked(_) => {
                            mouse_clicked_candidates.push((*id, element.z_value(), msg))
                        }
                        _ => hashmap_vec_push(&mut self.messages, *id, msg),
                    };
                });
            });
        }

        let max_z_element = mouse_hover_candidates.iter().max_by(|a, b| a.1.cmp(&b.1));

        if let Some((max_z_id, max_z, max_msg)) = max_z_element {
            hashmap_vec_push(&mut self.messages, *max_z_id as ElementId, max_msg.clone());

            let next_element = mouse_hover_candidates
                .iter()
                .filter(|(_, z, _)| z != max_z)
                .max_by(|a, b| a.1.cmp(&b.1));

            if let Some((next_id, _, next_msg)) = next_element {
                match max_msg {
                    Message::MouseEntered if next_msg == &Message::MouseInside => {
                        Some(Message::MouseLeft)
                    }
                    Message::MouseLeft if next_msg != &Message::MouseLeft => {
                        Some(Message::MouseEntered)
                    }
                    _ => None,
                }
                .map(|msg| hashmap_vec_push(&mut self.messages, *next_id as ElementId, msg));
            }
        }

        mouse_clicked_candidates
            .into_iter()
            .max_by(|a, b| a.1.cmp(&b.1))
            .map(|(id, _z, msg)| {
                hashmap_vec_push(&mut self.messages, id, msg);
            });
    }

    pub fn render(&mut self) {
        self.actions.clear();
        self.update_elements();
        self.render_elements();
    }

    fn render_elements(&self) {
        let mut sorted_elements: Vec<&Box<dyn Element>> = self.elements.values().collect();
        sorted_elements.sort_by(|a, b| a.z_value().cmp(&b.z_value()));
        sorted_elements.iter().for_each(|e| e.render());
    }

    fn update_elements(&mut self) {
        let mut target_messages: HashMap<ElementId, NormalizedMessages> = HashMap::new();

        // convert from (source, Vec<Message>) to (target, NormalizedMessages)
        for (source_id, msg) in self.messages.drain() {
            self.subscribers.get(&source_id).map(|subscribers| {
                subscribers.iter().for_each(|subscriber_id| {
                    target_messages.entry(*subscriber_id).or_default();
                    msg.iter().for_each(|m| {
                        target_messages.get_mut(&subscriber_id).unwrap().insert(m);
                    });
                })
            });
        }

        // update targets
        for (target_id, normalized) in target_messages {
            for m in normalized.messages() {
                let action = self.elements.get_mut(&target_id).unwrap().update(&m);
                action.map(|a| self.actions.push(a));
            }
        }
    }

    pub fn add_subscriber(&mut self, source: ElementId, subscriber: ElementId) {
        self.subscribers.entry(source).or_default();
        self.subscribers.get_mut(&source).unwrap().push(subscriber);
    }
}

#[derive(Default)]
struct NormalizedMessages {
    // HashSet would have been better, but Message supports no Eq
    messages: Vec<Message>,
}

impl NormalizedMessages {
    fn new() -> Self {
        Self {
            messages: Vec::new(),
        }
    }

    fn insert(&mut self, message: &Message) {
        if self.messages.contains(message) {
            return;
        }
        if message == &Message::MouseEntered && self.messages.contains(&Message::MouseLeft) {
            self.messages.retain(|e| e != &Message::MouseLeft);
            return;
        }
        if message == &Message::MouseLeft && self.messages.contains(&Message::MouseEntered) {
            self.messages.retain(|e| e != &Message::MouseEntered);
            return;
        }
        self.messages.push(message.clone());
    }

    fn messages(&self) -> &Vec<Message> {
        &self.messages
    }
}
