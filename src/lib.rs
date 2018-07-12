extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate fnv;

pub mod raw;
pub mod statistics;

use fnv::FnvHashMap;

use serde::ser::Serialize;
use serde::ser::Serializer;

use std::cmp;
use std::fmt;
use std::u64;
use std::i64;

use raw::CombatEvent;
use raw::CombatStateChange;
use raw::HitResult;
use raw::Skill;

/// The type of profession, includes NPCs and Gadgets
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize)]
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
    pub fn core_profession(self) -> Profession {
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

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize)]
pub enum Boss {
    ValeGuardian,
    Gorseval,
    Sabetha,
    Slothasor,
    Matthias,
    KeepConstruct,
    Xera,
    Cairn,
    MursaatOverseer,
    Samarog,
    Deimos,
    SoullessHorror,
    Dhuum,
    Unknown,
}

impl Boss {
    fn from_species_id(species: u16) -> Boss {
        match species {
            0x3c4e => Boss::ValeGuardian,
            0x3c45 => Boss::Gorseval,
            0x3c0f => Boss::Sabetha,
            0x3efb => Boss::Slothasor,
            0x3ef3 => Boss::Matthias,
            0x3f6b => Boss::KeepConstruct,
            0x3f76 => Boss::Xera,
            // ???
            // 0x3f9e => Boss::
            0x432a => Boss::Cairn,
            0x4314 => Boss::MursaatOverseer,
            0x4324 => Boss::Samarog,
            0x4302 => Boss::Deimos,
            0x4d37 => Boss::SoullessHorror,
            0x4bfa => Boss::Dhuum,
            _      => Boss::Unknown,
        }
    }
}


/// A game actor present in the encounter
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
    #[inline(always)]
    pub fn name(&self) -> &str {
        self.inner.name()
    }

    #[inline(always)]
    pub fn account_name(&self) -> &str {
        self.inner.account_name()
    }

    #[inline(always)]
    pub fn subgroup(&self) -> &str {
        self.inner.subgroup()
    }

    #[inline(always)]
    pub fn profession(&self) -> Profession {
        self.inner.profession()
    }

    #[inline(always)]
    pub fn species_id(&self) -> Option<u16> {
        self.inner.species_id()
    }

    #[inline]
    pub fn is_player_character(&self) -> bool {
        match self.inner.profession() {
            Profession::Gadget | Profession::NonPlayableCharacter => false,
            _ => true,
        }
    }

    /// Returns true if the agent died during the encounter
    #[inline(always)]
    pub fn did_die(&self) -> bool {
        self.meta.died
    }

    #[inline(always)]
    pub fn first_aware(&self) -> u64 {
        self.meta.first_aware
    }

    #[inline(always)]
    pub fn last_aware(&self) -> u64 {
        self.meta.last_aware
    }
}

impl Serialize for Agent {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
      where S: Serializer {
        use serde::ser::SerializeMap;

        let mut map = serializer.serialize_map(Some(10))?;

        map.serialize_entry("name",        self.inner.name())?;
        map.serialize_entry("accountName", self.inner.account_name())?;
        map.serialize_entry("subgroup",    self.inner.subgroup())?;
        map.serialize_entry("isPlayer",    &self.is_player_character())?;
        map.serialize_entry("speciesId",   &self.species_id())?;
        map.serialize_entry("profession",  &self.profession())?;
        map.serialize_entry("firstAware",  &self.meta.first_aware)?;
        map.serialize_entry("lastAware",   &self.meta.last_aware)?;
        map.serialize_entry("isPov",       &self.meta.is_pov)?;
        map.serialize_entry("didDie",      &self.did_die())?;

        map.end()
    }
}

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
    // If the agent died
    died:          bool,
    // If this agent is the point of view
    is_pov:        bool,
}

impl Default for AgentMetadata {
    fn default() -> Self {
        AgentMetadata {
            instid:        InstanceId::empty(),
            first_aware:   0,
            last_aware:    u64::MAX,
            master_instid: InstanceId::empty(),
            master_agent:  AgentId::empty(),
            died:          false,
            is_pov:        false,
        }
    }
}

#[derive(Debug)]
pub struct Metadata<'a> {
    buffer: &'a raw::EvtcBuf<'a>,
    agents: Vec<Agent>,
    start:  u64,
    end:    u64,
}

impl<'a> Metadata<'a> {
    pub fn new(buffer: &'a raw::EvtcBuf) -> Self {
        let mut map   = FnvHashMap::<AgentId, AgentMetadata>::with_capacity_and_hasher(buffer.agents.len(), Default::default());
        let mut start = u64::MAX;
        let mut end   = 0;

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
                died:          false,
                is_pov:        false,
            });

            // Apparently if it is not a combat-state-change then it is wrong
            if e.event_type() == EventType::StateChange {
                meta.instid = e.source_instance();

                if e.state_change() == CombatStateChange::PointOfView {
                    meta.is_pov = true;
                }
            }

            meta.last_aware = e.time();

            if e.master_source_instance() != InstanceId::empty() {
                meta.master_instid = e.master_source_instance();
                meta.master_agent  = master_agent.unwrap_or(meta.master_agent);
            }

            if e.state_change() == CombatStateChange::ChangeDead {
                meta.died = true;
            }

            start = cmp::min(start, e.time());
            end   = cmp::max(end, e.time());
        }

/*
        for v in map.values().filter(|v| (v.master_instid != InstanceId::empty()) ^ (v.master_agent != AgentId::empty())) {
            // FIXME: Is this necessary?
            println!("{:?}", v);
        }
        */

        // TODO: Filter agents?
        Metadata {
            buffer,
            agents: buffer.agents.iter().map(|agent| Agent {
                inner: *agent,
                meta:  map.get(&{agent.id}).map(|m| m.clone()).unwrap_or(Default::default()),
            }).collect(),
            start,
            end,
        }
    }

    pub fn agents(&self) -> &[Agent] {
        &self.agents
    }

    pub fn bosses(&self) -> impl Iterator<Item=&Agent> {
        let boss_id = self.buffer.header.boss_id;
        
        self.agents.iter().filter(move |a| a.species_id() == Some(boss_id))
    }

    pub fn boss(&self) -> Boss {
        Boss::from_species_id(self.buffer.header.boss_id)
    }

    /// Only returns the events which happened while the boss(es) were present in the fight,
    /// does not contain gaps.
    pub fn encounter_events(&self) -> impl Iterator<Item=&CombatEvent> {
        let (start, end) = self.bosses().fold((u64::MAX, 0), |(start, end), a| (cmp::min(start, a.first_aware()), cmp::max(end, a.last_aware())));

        self.buffer.events.iter().filter(move |e| start <= e.time() && e.time() <= end)
    }

    pub fn skills(&self) -> impl Iterator<Item=&Skill> {
        // TODO: There seem to be kinda empty skills in this list
        self.buffer.skills.iter().chain(raw::UNLISTED_SKILLS.iter())
    }

    pub fn skill_list(&self) -> SkillList {
        SkillList {
            skills: self.buffer.skills
        }
    }

    pub fn agents_for_master(&self, a: &Agent) -> impl Iterator<Item=&Agent> {
        let master_id = a.meta.instid;

        self.agents.iter().filter(move |a| a.meta.master_instid == master_id)
    }

    #[inline]
    pub fn log_start(&self) -> u64 {
        self.start
    }

    #[inline]
    pub fn log_end(&self) -> u64 {
        self.end
    }
}

#[derive(Clone, Copy, Debug)]
pub struct SkillList<'a> {
    skills: &'a [Skill],
}

impl<'a> Serialize for SkillList<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
      where S: Serializer {
        use serde::ser::SerializeMap;

        let mut map = serializer.serialize_map(Some(self.skills.len()))?;

        for s in self.skills.iter().filter(|s| s.id != 0) {
            map.serialize_entry(&{s.id}, s.name())?;
        }

        map.end()
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

    /// Returns the damage done by this event
    /// 
    /// Normalized across physical hits and buff events.
    #[inline]
    fn damage(&self) -> i64;

    #[inline]
    fn hit_result(&self) -> HitResult;

    #[inline]
    fn is_source_flanking(&self) -> bool;

    #[inline]
    fn is_source_moving(&self) -> bool;

    // TODO: Maybe option?
    #[inline]
    fn buff_damage(&self) -> i64;

    #[inline]
    fn state_change(&self) -> CombatStateChange;

    #[inline]
    fn is_source_over90(&self) -> bool;

    #[inline]
    fn skill_id(&self) -> u16;

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

    fn from_agent_and_gadgets(&self, agent: &Agent) -> bool {
        match self.state_change() {
            CombatStateChange::LogStart
            | CombatStateChange::LogEnd
            | CombatStateChange::Language
            | CombatStateChange::ShardId
            | CombatStateChange::GwBuild => false,
            _ => self.source_agent() == {agent.inner.id} || self.master_source_instance() == agent.meta.instid,
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

    #[inline]
    fn is_physical_hit(&self) -> bool {
        self.event_type() == EventType::PhysicalHit
    }
    
    #[inline]
    fn is_condition_tick(&self) -> bool {
        self.event_type() == EventType::BuffApplication && self.buff_damage() > 0
    }

    #[inline]
    fn is_damage(&self) -> bool {
        self.is_physical_hit() || self.is_condition_tick()
    }
}

#[derive(Debug, Clone, Serialize)]
struct TimeEntry {
    /// Timestamp, rounded to seconds, in microseconds
    time:        u64,
    /// Health fraction, scaled 10000x
    health:      Option<u64>,
    /// Damage done during this second
    damage:      Option<i64>,
    /// Average DPS
    dps:         Option<i64>,
    boss_dps:    Option<i64>,
    /// If the player got downed this second
    downed:      bool,
    /// If the player swapped weapon this second
    weapon_swap: bool,
}

impl TimeEntry {
    #[inline]
    fn with_time(t: u64) -> Self {
        TimeEntry {
            time:        t,
            health:      None,
            damage:      None,
            dps:         None,
            boss_dps:    None,
            downed:      false,
            weapon_swap: false,
        }
    }

    #[inline]
    fn has_data(&self) -> bool {
           self.health.is_some()
        || self.damage.is_some()
        || self.dps.is_some()
        || self.boss_dps.is_some()
        || self.downed
        || self.weapon_swap
    }
}

/// The time-series data for a player
#[derive(Debug, Clone)]
pub struct TimeSeries {
    series: Vec<TimeEntry>,
}

impl TimeSeries {
    pub fn new(meta: &Metadata) -> Self {
        assert!(meta.end > meta.start);

        TimeSeries {
            series: Vec::with_capacity(((meta.end - meta.start) / 1000) as usize),
        }
    }

    pub fn parse_agent(meta: &Metadata, agent: &Agent) -> Self {
        let mut series = Self::new(meta);

        series.parse(meta.encounter_events().filter(|e| e.from_agent_and_gadgets(agent)));

        series
    }

    #[inline]
    pub fn parse<'a, E: 'a + Event, I: Iterator<Item=&'a E>>(&mut self, mut iter: I) {
        let mut entry = if let Some(event) = iter.next() {
            let mut entry = TimeEntry::with_time(event.time() / 1000);

            self.parse_item(&mut entry, event);

            entry
        }
        else {
            return;
        };

        for e in iter {
            if entry.time != e.time() / 1000 {
                if entry.has_data() {
                    self.series.push(entry);
                }

                entry = TimeEntry::with_time(e.time() / 1000);
            }

            self.parse_item(&mut entry, e);
        }
    }

    fn parse_item<'a, E: 'a + Event>(&mut self, entry: &mut TimeEntry, event: &E) {
        match (event.state_change(),) {
            (CombatStateChange::ChangeDown,)   => entry.downed      = true,
            (CombatStateChange::HealthUpdate,) => entry.health      = Some(event.target_agent().0),
            (CombatStateChange::WeapSwap,)     => entry.weapon_swap = true,
            _ => {},
        }
    }
}

impl Serialize for TimeSeries {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
      where S: Serializer {
        self.series.serialize(serializer)
    }
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