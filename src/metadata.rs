use event::Buff;
use event::Event;
use event::Meta;
use event::MetaEventData;
use event::Source;
use event::StateChange;

use fnv::FnvHashMap;

use serde::ser::Serialize;
use serde::ser::Serializer;

use std::u32;
use std::u64;
use std::cmp;
use std::fmt;

use event::raw::Agent as RawAgent;
use event::raw::EvtcBuf;
use event::raw::Language;
use event::raw::Skill;
use event::raw::UNLISTED_SKILLS;
use event::raw::CombatEventV1;

use AgentId;
use Boss;
use InstanceId;
use Profession;
use SpeciesId;

/// A game agent present in the encounter
#[derive(Debug, Clone)]
pub struct Agent {
    // Agent address
    inner: RawAgent,
    meta:  AgentMetadata,
}

impl fmt::Display for Agent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} ({}) {} {} [t={} h={} c={}]", self.inner.name(), self.inner.account_name(), self.inner.profession(), self.inner.subgroup(), {self.inner.toughness}, {self.inner.healing}, {self.inner.condition_dmg})
    }
}

impl PartialEq for Agent {
    fn eq(&self, other: &Agent) -> bool {
        self.inner.id() == other.inner.id()
    }
}

impl Agent {
    #[inline(always)]
    pub fn id(&self) -> AgentId {
        self.inner.id()
    }

    #[inline(always)]
    pub fn instance_id(&self) -> InstanceId {
        self.meta.instid
    }

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

    /// Returns the time of death if the agent died during the encounter
    #[inline(always)]
    pub fn died(&self) -> Option<u64> {
        self.meta.died
    }

    /// Returns true if the agent died during the encounter
    #[inline(always)]
    pub fn did_die(&self) -> bool {
        self.meta.died.is_some()
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

        let mut map = serializer.serialize_map(Some(13))?;

        map.serialize_entry("name",          self.name())?;
        map.serialize_entry("accountName",   self.account_name())?;
        map.serialize_entry("subgroup",      self.subgroup())?;
        map.serialize_entry("speciesId",     &self.profession().species_id())?;
        map.serialize_entry("profession",    &self.profession())?;
        map.serialize_entry("toughness",     &{self.inner.toughness})?;
        map.serialize_entry("concentration", &{self.inner.concentration})?;
        map.serialize_entry("healing",       &{self.inner.healing})?;
        map.serialize_entry("conditionDmg",  &{self.inner.condition_dmg})?;
        map.serialize_entry("firstAware",    &self.first_aware())?;
        map.serialize_entry("lastAware",     &self.last_aware())?;
        map.serialize_entry("isPov",         &self.meta.is_pov)?;
        map.serialize_entry("diedAt",        &self.meta.died)?;

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
    died:          Option<u64>,
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
            died:          None,
            is_pov:        false,
        }
    }
}

const EXTRA_BOSS_IDS: &'static [(SpeciesId, Profession)] = &[
    // Xera:
    (SpeciesId(16246), Profession::NonPlayableCharacter(SpeciesId(16286))),
    // Deimos:
    (SpeciesId(17154), Profession::Gadget(SpeciesId(8467))),
    (SpeciesId(17154), Profession::Gadget(SpeciesId(8471))),
];

#[derive(Debug)]
pub struct Metadata<'a> {
    buffer: &'a EvtcBuf<'a>,
    agents: Vec<Agent>,
    start:  u32,
    end:    u32,
    lang:   Language,
    build:  u64,
    shard:  u64,
}

impl<'a> Metadata<'a> {
    pub fn new(buffer: &'a EvtcBuf) -> Self {
        let mut map   = FnvHashMap::<AgentId, AgentMetadata>::with_capacity_and_hasher(buffer.agents.len(), Default::default());
        let mut start = u32::MAX;
        let mut end   = 0;
        let mut shard = 0;
        let mut build = 0;
        let mut lang  = Language::English;

        // Determine meta stuff
        for e in buffer.events.iter().filter_map(Event::into_meta) {
            match e.into_enum() {
                // TODO: Save the extra values
                MetaEventData::LogStart { server, .. } => start = server,
                MetaEventData::LogEnd   { server, .. } => end   = server,
                MetaEventData::Language(l)             => lang  = l,
                MetaEventData::Gw2Build(b)             => build = b,
                MetaEventData::ShardId(s)              => shard = s,
            }
        }

        for e in buffer.events.iter().filter_map(Event::into_source) {
            let master_agent = e.master_instance()
                                .and_then(|i| map.iter().find(|(_id, m)| m.instid == i).map(|(&id, _)| id));
            if let Some(_) = e.master_instance() {
                if let None = master_agent {
                    panic!("{:?} {:?}", e, map.iter().map(|(_, a)| a.instid).collect::<Vec<_>>());
                }
            }

            let mut meta = map.entry(e.agent()).or_insert(AgentMetadata {
                instid:        InstanceId::empty(),
                first_aware:   e.time(),
                last_aware:    e.time(),
                master_instid: InstanceId::empty(),
                master_agent:  AgentId::empty(),
                died:          None,
                is_pov:        false,
            });

            match e.state_change() {
                Some(StateChange::EnterCombat(_)) |
                  Some(StateChange::MaxHealthUpdate(_)) |
                  Some(StateChange::Spawn)     => meta.instid = e.instance(),
                Some(StateChange::PointOfView) => meta.is_pov = true,
                // TODO: For players, revert this if death is after *all* of the boss deaths
                Some(StateChange::ChangeDead)  => meta.died   = Some(e.time()),
                // Xera
                // Second one
                Some(StateChange::Despawn)     => meta.died   = Some(e.time()),
                _                              => if meta.instid == InstanceId::empty() {
                    // FIXME: This seems to be a bit heavy-handed, sometimes EnterCombat and the like do not happen
                    meta.instid = e.instance();
                },
            }

            if let Some(b) = e.clone().into_buff() {
                match b.skill() {
                    // Xera: First one becomes invulnerable using a skill
                    762   => meta.died = Some(e.time()),
                    34113 => meta.died = Some(e.time()),
                    _     => {},
                }
            }

            meta.last_aware = e.time();

            if let Some(i) = e.master_instance() {
                meta.master_instid = i;
                meta.master_agent  = master_agent.unwrap_or(meta.master_agent);
            }
        }

        // TODO: Filter agents?
        Metadata {
            buffer,
            agents: buffer.agents.iter().map(|agent| Agent {
                inner: *agent,
                meta:  map.get(&{agent.id}).map(|m| m.clone()).unwrap_or(Default::default()),
            }).collect(),
            start,
            end,
            lang,
            build,
            shard,
        }
    }

    pub fn agents(&self) -> &[Agent] {
        &self.agents
    }

    pub fn bosses(&self) -> impl Iterator<Item=&Agent> {
        let boss_id = self.buffer.header.boss_id;

        self.agents.iter().filter(move |a| a.profession() == Profession::NonPlayableCharacter(boss_id) ||
            EXTRA_BOSS_IDS.iter()
                          .map(|(id, other)| if boss_id == *id { *other == a.profession() } else { false })
                          .fold(false, |a, b| a || b))
    }

    pub fn boss(&self) -> Boss {
        Boss::from_species_id(self.buffer.header.boss_id)
    }

    /// Only returns the events which happened while the boss(es) were present in the fight,
    /// does not contain gaps.
    pub fn encounter_events(&'a self) -> impl 'a + Iterator<Item=&'a CombatEventV1> {
        // TODO: Move to method
        let (start, end) = self.bosses().fold((u64::MAX, 0), |(start, end), a| (cmp::min(start, a.first_aware()), cmp::max(end, a.last_aware())));

        self.buffer.events.iter().filter(move |e| start <= e.time() && e.time() <= end)
    }

    pub fn skills(&self) -> impl Iterator<Item=&Skill> {
        // TODO: There seem to be kinda empty skills in this list
        self.buffer.skills.iter().chain(UNLISTED_SKILLS.iter())
    }

    pub fn skill_list(&self) -> SkillList {
        SkillList {
            skills: self.buffer.skills
        }
    }

    pub fn agents_for_master(&self, a: &Agent) -> impl Iterator<Item=&Agent> {
        let master_id = a.id();

        self.agents.iter().filter(move |a| a.meta.master_agent == master_id)
    }

    #[inline]
    pub fn log_start(&self) -> u32 {
        self.start
    }

    #[inline]
    pub fn log_end(&self) -> u32 {
        self.end
    }

    #[inline]
    pub fn language(&self) -> Language {
        self.lang
    }

    #[inline]
    pub fn server_shard(&self) -> u64 {
        self.shard
    }

    #[inline]
    pub fn game_build(&self) -> u64 {
        self.build
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

        for s in self.skills.iter().filter(|s| s.id() != 0) {
            map.serialize_entry(&s.id(), s.name())?;
        }

        map.end()
    }
}