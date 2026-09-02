use std::{collections::HashMap, fmt};

use serde::de::{Error, MapAccess, SeqAccess, Visitor};

use super::{EventConfig, On};

/// Reads the `on:` entry of a workflow, which may be a single event name, a
/// sequence of event names, or a mapping of event names to their filters.
pub struct OnVisitor;

impl<'de> Visitor<'de> for OnVisitor {
    type Value = On;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a string, sequence of strings, or mapping of event configs")
    }

    fn visit_str<E: Error>(self, event: &str) -> Result<On, E> {
        Ok(On::Single(event.to_owned()))
    }

    fn visit_string<E: Error>(self, event: String) -> Result<On, E> {
        Ok(On::Single(event))
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut sequence: A) -> Result<On, A::Error> {
        let mut events = Vec::new();
        while let Some(event) = sequence.next_element::<String>()? {
            events.push(event);
        }
        Ok(On::Multiple(events))
    }

    fn visit_map<M: MapAccess<'de>>(self, mut map: M) -> Result<On, M::Error> {
        let mut events = HashMap::new();
        while let Some((event, config)) = map.next_entry::<String, Option<EventConfig>>()? {
            events.insert(event, config);
        }
        Ok(On::WithTypes(events))
    }
}
