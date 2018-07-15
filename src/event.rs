use Agent;

use types::AgentId;
use types::InstanceId;
use raw::Language;

#[derive(Debug, Copy, Clone)]
pub struct Event {
    pub time:  u64,
    pub event: EventType,
}

impl Event {
    #[inline]
    pub fn from_agent_and_gadgets(&self, agent: &Agent) -> bool {
        self.event.from_agent_and_gadgets(agent)
    }
}

#[derive(Debug, Copy, Clone)]
pub enum EventType {
    /// Server unix timestamp, local unix timestamp, arcdpsId
    LogStart { server: u32, local: u32, arcdps_id: u64 },
    LogEnd   { server: u32, local: u32, arcdps_id: u64 },
    Language(Language),
    Gw2Build(u64),
    ShardId(u64),
    Agent {
        agent:           AgentId,
        instance:        InstanceId,
        master_instance: Option<InstanceId>,
        event:           AgentEvent,
    },
}

impl EventType {
    #[inline]
    pub fn from_agent_and_gadgets(&self, agent: &Agent) -> bool {
        match self {
            EventType::Agent { agent: a, master_instance: i, .. } => *a == agent.id() || *i == Some(agent.instance_id()),
            _ => false,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum AgentEvent {
    /// Entered combat in subgroup
    EnterCombat(u64),
    ExitCombat,
    ChangeUp,
    ChangeDead,
    ChangeDown,
    Spawn,
    Despawn,
    /// Agent has a health-update, value is % * 10000 (eg. 99.5% will be 9950)
    HealthUpdate(u64),
    WeaponSwap,
    MaxHealthUpdate(u64),
    PointOfView,
    /// Wiggly boxes, reward id and reward type
    Reward(u64, u32),
    /// Happens once per agent on start
    // TODO: What is this? Should have more data
    BuffInitial,
    Position { x: f32, y: f32, z: f32 },
    Velocity { x: f32, y: f32, z: f32 },
    /// Skill casts
    Activation { skill: u16, cast: Activation },

    WithTarget { agent: AgentId, instance: InstanceId, event: TargetEvent },
}

#[derive(Debug, Copy, Clone)]
pub enum TargetEvent {
    Buff(u16, Buff),
    Damage {
        skill:      u16,
        damage:     i64,
        flanking:   bool,
        moving:     bool,
        src_over90: bool,
        hit_type:   HitType,
    },
}

#[derive(Debug, Copy, Clone)]
pub enum Activation {
    // Normal cast, expected duration
    Normal(u32),
    // Fast cast (+50%), expected duration
    Quickness(u32),
    // Canceled but started channel, actual duration
    CancelFire(u32),
    // Canceled before channel, actual duration
    Cancel(u32),
    Reset,
}

#[derive(Debug, Copy, Clone)]
pub enum Buff {
    RemoveAll,
    RemoveSingle,
    // FIXME: Add more info about number of stacks and duration
    Application,
}

#[derive(Debug, Copy, Clone)]
pub enum HitType {
    Condi,
    Normal, 
    Crit, 
    Glance, 
    Block, 
    Evade, 
    Interrupt, 
    Absorb, 
    Blind, 
    KillingBlow, 
}

impl HitType {
    pub fn is_zero(self) -> bool {
        match self {
            HitType::Block | HitType::Evade | HitType::Interrupt | HitType::Absorb | HitType::Blind => true,
            _ => false,
        }
    }
}