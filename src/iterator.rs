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

    fn from_any_of<'a>(self, agents: &'a [&'a Agent]) -> FromAgentsIterator<'a, Self> {
        FromAgentsIterator {
            agents,
            inner: self,
        }
    }

    #[inline]
    fn targeting<'a>(self, agent: &'a Agent) -> TargetingAgentIterator<Self> {
        TargetingAgentIterator {
            agent_id: agent.id(),
            inner:    self,
        }
    }

    #[inline]
    fn related_to<'a>(self, agent: &'a Agent) -> RelatingToAgentIterator<Self> {
        RelatingToAgentIterator {
            agent_id: agent.id(),
            instance: agent.instance_id(),
            inner:    self,
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

pub struct FromAgentsIterator<'a, I> {
    agents: &'a [&'a Agent],
    inner:  I,
}

impl<'a, I: Iterator<Item=T>, T: Event> Iterator for FromAgentsIterator<'a, I> {
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
pub struct TargetingAgentIterator<I> {
    agent_id: AgentId,
    inner:    I,
}

impl<I: Iterator<Item=T>, T: Event> Iterator for TargetingAgentIterator<I> {
    type Item = T::TargetEvent;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(e) = self.inner.next() {
            if let Some(e) = e.targeting_agent(self.agent_id) {
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
        }

        None
    }
}

pub struct RelatingToAgentIterator<I> {
    agent_id: AgentId,
    instance: InstanceId,
    inner:    I,
}

impl<I: Iterator<Item=T>, T: Event> Iterator for RelatingToAgentIterator<I> {
    type Item = T::SourceEvent;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(e) = self.inner.next() {
            if let Some(e) = e.into_source() {
                // TODO: Maybe improve efficiency?
                if let Some(e) = e.clone().from_agent_or_gadgets(self.agent_id, self.instance) {
                    return Some(e);
                }

                if let Some(_) = e.clone().targeting_agent(self.agent_id) {
                    // This should always work
                    return Some(e);
                }
            }
        }

        None
    }
}