
pub mod raw;

use std::u64;
use std::fmt;
use std::collections::HashMap;

use raw::CombatStateChange;
use raw::IFF;

/*
// TODO: Group the common stuff
pub struct Event {
    time:      u64,
    src_agent: u64,
    dst_agent: u64,
}

impl Event {
    fn from_combat_event(e: raw::CombatEvent) -> Self {
        let type = e.event_type();

        Event {
            time:      e.time,
            // For buff-remove events, the target is the source and reverse
            src_agent: if type == raw::EventType::BuffRemove { e.dst_agent } else { e.src_agent },
            dst_agent: if type == raw::EventType::BuffRemove { e.src_agent } else { e.dst_agent },
        }
    }
}
*/

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum Profession {
    Gadget,
    NonPlayableCharacter,
    Guardian,
    Warrior,
    Engineer,
    Ranger,
    Thief,
    Elementalist,
    Mesmer,
    Necromancer,
    Revenant,
    Dragonhunter,
    Berserker,
    Scrapper,
    Druid,
    Daredevil,
    Tempest,
    Chronomancer,
    Reaper,
    Herald,
    Soulbeast,
    Weaver,
    Holosmith,
    Deadeye,
    Mirage,
    Scourge,
    Spellbreaker,
    Firebrand,
    Renegade,
    Unknown,
}

impl Profession {
    fn core_profession(self) -> Profession {
        match self {
            Profession::Dragonhunter => Profession::Guardian,
            Profession::Firebrand    => Profession::Guardian,
            Profession::Berserker    => Profession::Warrior,
            Profession::Spellbreaker => Profession::Warrior,
            Profession::Herald       => Profession::Revenant,
            Profession::Renegade     => Profession::Revenant,
            Profession::Scrapper     => Profession::Engineer,
            Profession::Holosmith    => Profession::Engineer,
            Profession::Druid        => Profession::Ranger,
            Profession::Soulbeast    => Profession::Ranger,
            Profession::Daredevil    => Profession::Thief,
            Profession::Deadeye      => Profession::Thief,
            Profession::Tempest      => Profession::Elementalist,
            Profession::Weaver       => Profession::Elementalist,
            Profession::Chronomancer => Profession::Mesmer,
            Profession::Mirage       => Profession::Mesmer,
            Profession::Reaper       => Profession::Necromancer,
            Profession::Scourge      => Profession::Necromancer,
            x => x,
        }
    }
}

impl fmt::Display for Profession {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

#[derive(Debug, Clone)]
pub struct Agent {
    // Agent address
    inner: raw::Agent,
    meta:  AgentMetadata,
}

impl fmt::Display for Agent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} ({}) {} {} [t={} h={} c={}]", self.inner.name(), self.inner.account_name(), self.inner.profession(), self.inner.subgroup(), {self.inner.toughness}, {self.inner.healing}, {self.inner.condition})
    }
}

impl PartialEq for Agent {
    fn eq(&self, other: &Agent) -> bool {
        self.inner.id() == other.inner.id()
    }
}

impl Agent {
    pub fn name(&self) -> &str {
        self.inner.name()
    }

    pub fn account_name(&self) -> &str {
        self.inner.account_name()
    }

    pub fn subgroup(&self) -> &str {
        self.inner.subgroup()
    }

    pub fn proffession(&self) -> Profession {
        self.inner.profession()
    }

    pub fn is_player_character(&self) -> bool {
        match self.inner.profession() {
            Profession::Gadget | Profession::NonPlayableCharacter => false,
            _ => true,
        }
    }
}

/*
impl<I> EventsFilter for I
  where I: Iterator<Item=&raw::CombatEvent> {
}
*/

#[derive(Debug, Clone)]
struct AgentMetadata {
    // Agent instance id
    instid:        InstanceId,
    // Time when first observed
    first_aware:   u64,
    // Time when last observed
    last_aware:    u64,
    // Owning instance id
    master_instid: InstanceId,
    // Owning address
    master_agent:  AgentId,
}

impl Default for AgentMetadata {
    fn default() -> Self {
        AgentMetadata {
            instid:        InstanceId::empty(),
            first_aware:   0,
            last_aware:    u64::MAX,
            master_instid: InstanceId::empty(),
            master_agent:  AgentId::empty(),
        }
    }
}

#[derive(Debug)]
pub struct Metadata {
    agents: Vec<Agent>,
    // agent_data: HashMap<u64, AgentMetadata>,
}

impl Metadata {
    pub fn new(buffer: &raw::EvtcBuf) -> Self {
        let mut map = HashMap::<AgentId, AgentMetadata>::with_capacity(buffer.agents.len());

        for e in buffer.events.iter() {
            let master_agent = if e.master_source_instance() != InstanceId::empty() {
                // TODO: Maybe check so our parent hasn't died yet? idk
                // FIXME: This does not seem to work properly
                map.iter().find(|(_id, m)| m.instid == e.master_source_instance() /*&& m.first_aware < e.time*/).map(|(&id, _)| id)
            } else { None };

            let mut meta = map.entry(e.source_agent()).or_insert(AgentMetadata {
                instid:        InstanceId::empty(),
                first_aware:   e.time(),
                last_aware:    e.time(),
                master_instid: InstanceId::empty(),
                master_agent:  AgentId::empty(),
            });

            // Apparently if it is not a combat-state-change then it is wrong
            if e.event_type() == EventType::StateChange {
                meta.instid = e.source_instance();
            }

            meta.last_aware = e.time();

            if e.master_source_instance() != InstanceId::empty() {
                meta.master_instid = e.master_source_instance();
                meta.master_agent  = master_agent.unwrap_or(meta.master_agent);
            }
        }

        for v in map.values().filter(|v| (v.master_instid != InstanceId::empty()) ^ (v.master_agent != AgentId::empty())) {
            println!("{:?}", v);
        }

        Metadata {
            agents: buffer.agents.iter().map(|agent| Agent {
                inner: *agent,
                meta:  map.get(&{agent.id}).map(|m| m.clone()).unwrap_or(Default::default()),
            }).collect(),
        }
    }

    pub fn agents(&self) -> &[Agent] {
        &self.agents
    }
}

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum EventType {
    StateChange,
    Activation,
    BuffRemove,
    BuffApplication,
    PhysicalHit,
}

#[derive(Clone, Copy, Eq, PartialEq, Debug, Hash)]
pub struct AgentId(u64);

impl fmt::Display for AgentId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "#{}", self.0)
    }
}

impl Default for AgentId {
    fn default() -> Self {
        AgentId::empty()
    }
}

impl AgentId {
    #[inline(always)]
    pub fn empty() -> Self {
        AgentId(0)
    }
}

#[derive(Clone, Copy, Eq, PartialEq, Debug, Hash)]
pub struct InstanceId(u16);

impl fmt::Display for InstanceId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{{{}}}", self.0)
    }
}

impl Default for InstanceId {
    fn default() -> Self {
        InstanceId::empty()
    }
}

impl InstanceId {
    #[inline(always)]
    pub fn empty() -> Self {
        InstanceId(0)
    }
}

pub trait Event {
    #[inline]
    fn time(&self) -> u64;

    #[inline]
    fn event_type(&self) -> EventType;

    #[inline]
    fn source_agent(&self) -> AgentId;

    // TODO: Maybe option?
    #[inline]
    fn target_agent(&self) -> AgentId;

    // TODO: Maybe option?
    #[inline]
    fn target_instance(&self) -> InstanceId;

    // TODO: Maybe option?
    #[inline]
    fn source_instance(&self) -> InstanceId;

    // TODO: Maybe option?
    #[inline]
    fn master_source_instance(&self) -> InstanceId;

    // TODO

    // TODO: Typesafe this
    #[inline]
    fn value(&self) -> i64;

    // TODO: Maybe option?
    #[inline]
    fn buff_damage(&self) -> i64;

    #[inline]
    fn state_change(&self) -> CombatStateChange;

    #[inline]
    fn targeting_agent(&self, agent: &Agent) -> bool {
        match self.state_change() {
            CombatStateChange::EnterCombat
            | CombatStateChange::HealthUpdate
            | CombatStateChange::WeapSwap
            | CombatStateChange::MaxHealthUpdate
            | CombatStateChange::Reward
            | CombatStateChange::Position
            | CombatStateChange::Velocity => false,
            _ => self.target_agent() == {agent.inner.id},
        }
    }

    #[inline]
    fn from_agent(&self, agent: &Agent) -> bool {
        match self.state_change() {
            CombatStateChange::LogStart
            | CombatStateChange::LogEnd
            | CombatStateChange::Language
            | CombatStateChange::ShardId
            | CombatStateChange::GwBuild => false,
            _ => self.source_agent() == {agent.inner.id},
        }
    }

    #[inline]
    fn is_boon(&self) -> bool {
        match self.event_type() {
            EventType::BuffRemove      => true,
            EventType::BuffApplication => self.buff_damage() == 0,
            _                          => false
        }
    }

/*
    #[inline]
    fn is_physica_damage(&self) -> bool {
        self.target_instance() != InstanceId::empty() && self.event_type() == EventType::PhysicalHit && self.iff() == IFF::Foe
    }
    */
}

#[derive(Debug, Copy, Clone)]
pub struct Quickness([u64; 5]);

//impl specs::Component for Quickness {
//    type Storage = specs::VecStorage<Self>;
//}

impl Default for Quickness {
    fn default() -> Self {
        Quickness([0; 5])
    }
}

impl Quickness {
    pub fn decrease(&mut self, mut dtime: u64) {
        for i in (0..5).rev() {
            let d = dtime.saturating_sub(self.0[i]);

            self.0[i].saturating_sub(dtime);

            dtime = d;
        }

        self.0.sort();
    }

    pub fn increase(&mut self, time: u64) {
        for i in (0..5).rev() {
            if self.0[i] < time {
                self.0[i] = time;

                break;
            }
        }
    }

    pub fn stacks(&self) -> usize {
        self.0.iter().filter(|x| **x > 0).count()
    }
}

/*struct QuicknessSystem;

impl<'a> specs::System<'a> for QuicknessSystem {
    type SystemData = (specs::Fetch<'a, DeltaTime>, specs::WriteStorage<'a, Quickness>);

    fn run(&mut self, (dtime, mut quick): Self::SystemData) {
        for q in (&mut quick).join() {
            q.decrease(dtime.0);
        }
    }
}

struct QuicknessAdditionSystem(u16);

impl<'a> specs::System<'a> for QuicknessAdditionSystem {
    type SystemData = (specs::ReadStorage<'a, IncomingEvents>, specs::WriteStorage<'a, Quickness>);

    fn run(&mut self, (inc, mut quick): Self::SystemData) {
        for (i, q) in (&inc, &mut quick).join() {
            for e in &i.0 {
                if e.is_buff() && e.skill_id == self.0 {
                    q.increase(e.value as u64);
                }
            }
        }
    }
}
*/