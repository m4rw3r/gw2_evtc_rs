use Agent;
use AgentId;
use InstanceId;
use Event;

use std::iter::Iterator;

pub trait EventIteratorExt: Iterator
  where Self: Sized {
    #[inline]
    fn from_agent(self, agent: &Agent) -> FromAgentIterator<Self> {
        FromAgentIterator {
            agent_id: agent.id(),
            // instance: agent.instance_id(),
            inner:    self,
        }
    }

    #[inline]
    fn from_agent_or_gadgets(self, agent: &Agent) -> FromAgentGadgetsIterator<Self> {
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
}

impl<I: Iterator> EventIteratorExt for I {}

pub struct FromAgentIterator<I> {
    agent_id: AgentId,
    inner:    I,
}

impl<I: Iterator<Item=T>, T: Event> Iterator for FromAgentIterator<I> {
    type Item = T::SourceEvent;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(e) = self.inner.next() {
            if let Some(e) = e.from_agent(self.agent_id) {
                return Some(e);
            }
        }

        None
    }
}

pub struct FromAgentGadgetsIterator<I> {
    agent_id: AgentId,
    instance: InstanceId,
    inner:    I,
}

impl<I: Iterator<Item=T>, T: Event> Iterator for FromAgentGadgetsIterator<I> {
    type Item = T::SourceEvent;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(e) = self.inner.next() {
            if let Some(e) = e.from_agent_or_gadgets(self.agent_id, self.instance) {
                return Some(e);
            }
        }

        None
    }
}

pub struct FromAgents<'a, I> {
    agents: &'a [&'a Agent],
    inner:  I,
}

impl<'a, I: Iterator<Item=T>, T: Event> Iterator for FromAgents<'a, I> {
    type Item = T::SourceEvent;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(e) = self.inner.next() {
            if let Some(e) = e.from_any_of(self.agents.iter().map(|a| a.id())) {
                return Some(e);
            }
        }

        None
    }
}

pub struct TargetingAgents<'a, I> {
    agents: &'a [&'a Agent],
    inner:  I,
}

impl<'a, I: Iterator<Item=T>, T: Event> Iterator for TargetingAgents<'a, I> {
    type Item = T::TargetEvent;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(e) = self.inner.next() {
            if let Some(e) = e.targeting_any_of(self.agents.iter().map(|a| a.id())) {
                return Some(e);
            }
            /*
            if let Event { event: EventType::WithTarget { agent, instance, .. }, .. } = e {
                if self.agents.iter().any(|a| agent == a.id() || instance == a.instance_id()) {
                    return Some(e);
                }
            }
            */
        }

        None
    }
}