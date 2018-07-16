use Agent;
use AgentId;
use InstanceId;
use Event;
use EventType;
use TargetEvent;

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

pub struct FromAgentIterator<I> {
    agent_id: AgentId,
    inner:    I,
}

impl<I: Iterator<Item=Event>> Iterator for FromAgentIterator<I> {
    type Item = Event;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(e) = self.inner.next() {
            let Event { agent, .. } = e;

            if agent == self.agent_id {
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

impl<I: Iterator<Item=Event>> Iterator for FromAgentGadgetsIterator<I> {
    type Item = Event;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(e) = self.inner.next() {
            let Event { agent, master_instance, .. } = e;

            if agent == self.agent_id || master_instance == Some(self.instance) {
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

impl<'a, I: Iterator<Item=Event>> Iterator for FromAgents<'a, I> {
    type Item = Event;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(e) = self.inner.next() {
            let Event { agent, instance, .. } = e;

            if self.agents.iter().any(|a| agent == a.id() || instance == a.instance_id()) {
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

impl<'a, I: Iterator<Item=Event>> Iterator for TargetingAgents<'a, I> {
    type Item = Event;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(e) = self.inner.next() {
            if let Event { event: EventType::WithTarget { agent, instance, .. }, .. } = e {
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
        while let Some(Event { event: EventType::WithTarget { event, ..}, .. }) = self.inner.next() {
            return Some(event);
        }

        None
    }
}