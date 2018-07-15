use Agent;
use AgentId;
use InstanceId;
use Event;
use EventType;
use AgentEvent;
use TargetEvent;

use std::iter::Iterator;

pub trait EventIteratorExt: Iterator
  where Self: Sized {

    #[inline]
    fn from_agent_and_gadgets(self, agent: &Agent) -> FromAgentGadgetsIterator<Self> {
        FromAgentGadgetsIterator {
            agent_id: agent.id(),
            instance: agent.instance_id(),
            inner:    self,
        }
    }

    fn from_any_of<'a>(self, agents: &'a [&'a Agent]) -> FromAgents<'a, Self> {
        FromAgents {
            agents,
            inner: self,
        }
    }

    #[inline]
    fn targeting_any_of<'a>(self, agents: &'a [&'a Agent]) -> TargetingAgents<'a, Self> {
        TargetingAgents {
            agents,
            inner: self,
        }
    }

    #[inline]
    fn target_events(self) -> TargetEventIterator<Self> {
        TargetEventIterator {
            inner: self,
        }
    }
}

impl<I: Iterator> EventIteratorExt for I {}

pub struct FromAgentGadgetsIterator<I> {
    agent_id: AgentId,
    instance: InstanceId,
    inner:    I,
}

impl<I: Iterator<Item=Event>> Iterator for FromAgentGadgetsIterator<I> {
    type Item = Event;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(e) = self.inner.next() {
            if let EventType::Agent { agent: a, master_instance: i, .. } = e.event {
                if a == self.agent_id || i == Some(self.instance) {
                    return Some(e);
                }
            } 
        }

        None
    }
}

pub struct FromAgents<'a, I> {
    agents: &'a [&'a Agent],
    inner:  I,
}

impl<'a, I: Iterator<Item=Event>> Iterator for FromAgents<'a, I> {
    type Item = Event;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(e) = self.inner.next() {
            if let EventType::Agent { agent, instance, .. } = e.event {
                if self.agents.iter().any(|a| agent == a.id() || instance == a.instance_id()) {
                    return Some(e);
                }
            } 
        }

        None
    }
}

pub struct TargetingAgents<'a, I> {
    agents: &'a [&'a Agent],
    inner:  I,
}

impl<'a, I: Iterator<Item=Event>> Iterator for TargetingAgents<'a, I> {
    type Item = Event;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(e) = self.inner.next() {
            if let EventType::Agent { event: AgentEvent::WithTarget { agent, instance, .. }, .. } = e.event {
                if self.agents.iter().any(|a| agent == a.id() || instance == a.instance_id()) {
                    return Some(e);
                }
            } 
        }

        None
    }
}

pub struct TargetEventIterator<I> {
    inner:  I,
}

impl<I: Iterator<Item=Event>> Iterator for TargetEventIterator<I> {
    type Item = TargetEvent;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(e) = self.inner.next() {
            if let EventType::Agent { event: AgentEvent::WithTarget { event, ..}, .. } = e.event {
                return Some(event);
            } 
        }

        None
    }
}