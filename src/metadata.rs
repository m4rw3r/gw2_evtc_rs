use IntoEvent;
use Event;
use EventType;
use AgentEvent;
use TargetEvent;

use fnv::FnvHashMap;

use serde::ser::Serialize;
use serde::ser::Serializer;

use std::u32;
use std::u64;
use std::cmp;
use std::fmt;
use std::mem;

use raw::Agent as RawAgent;
use raw::CombatEvent;
use raw::CombatStateChange;
use raw::EvtcBuf;
use raw::Language;
use raw::Skill;
use raw::UNLISTED_SKILLS;

use types::AgentId;
use types::Boss;
// use types::EventType;
use types::InstanceId;
use types::Profession;
use types::SpeciesId;

/// A game actor present in the encounter
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

    #[inline(always)]
    pub fn species_id(&self) -> Option<SpeciesId> {
        self.inner.species_id()
    }

    #[inline]
    pub fn is_player_character(&self) -> bool {
        match self.inner.profession() {
            Profession::Gadget | Profession::NonPlayableCharacter => false,
            _ => true,
        }
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

        let mut map = serializer.serialize_map(Some(14))?;

        map.serialize_entry("name",          self.inner.name())?;
        map.serialize_entry("accountName",   self.inner.account_name())?;
        map.serialize_entry("subgroup",      self.inner.subgroup())?;
        map.serialize_entry("isPlayer",      &self.is_player_character())?;
        map.serialize_entry("speciesId",     &self.species_id())?;
        map.serialize_entry("profession",    &self.profession())?;
        map.serialize_entry("toughness",     &{self.inner.toughness})?;
        map.serialize_entry("concentration", &{self.inner.concentration})?;
        map.serialize_entry("healing",       &{self.inner.healing})?;
        map.serialize_entry("conditionDmg",  &{self.inner.condition_dmg})?;
        map.serialize_entry("firstAware",    &self.meta.first_aware)?;
        map.serialize_entry("lastAware",     &self.meta.last_aware)?;
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

        for e in buffer.events.iter().map(|e| e.to_event()) {
            match e.event {
                // TODO: Save the extra values
                EventType::LogStart { server, local, arcdps_id } => start = server,
                EventType::LogEnd   { server, local, arcdps_id } => end   = server,
                EventType::Language(l)                           => lang  = l,
                EventType::Gw2Build(b)                           => build = b,
                EventType::ShardId(s)                            => shard = s,
                EventType::Agent { agent, instance, master_instance, event: nested_event } => {
                    let master_agent = master_instance.and_then(|i| map.iter().find(|(_id, m)| m.instid == i).map(|(&id, _)| id));

                    let mut meta = map.entry(agent).or_insert(AgentMetadata {
                        instid:        InstanceId::empty(),
                        first_aware:   e.time,
                        last_aware:    e.time,
                        master_instid: InstanceId::empty(),
                        master_agent:  AgentId::empty(),
                        died:          None,
                        is_pov:        false,
                    });

                    match nested_event {
                        AgentEvent::EnterCombat(_) => meta.instid = instance,
                        AgentEvent::PointOfView    => meta.is_pov = true,
                        AgentEvent::ChangeDead     => meta.died   = Some(e.time),
                        _                          => {},
                    }

                    meta.last_aware = e.time;

                    if let Some(i) = master_instance {
                        meta.master_instid = i;
                        meta.master_agent  = master_agent.unwrap_or(meta.master_agent);
                    }
                }
            }

/*
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
                died:          None,
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

            match e.state_change() {
                CombatStateChange::ChangeDead => meta.died = Some(e.time()),
                CombatStateChange::LogStart   => start = e.value_as_time(),
                CombatStateChange::LogEnd     => end   = e.value_as_time(),
                // CombatStateChange::Language   => lang  = Language::from_agent_id(e.source_agent()),
                // FIXME: Do not use transmute, use the proper event conversion
                CombatStateChange::GwBuild    => build = unsafe { mem::transmute(e.source_agent()) },
                CombatStateChange::ShardId    => shard = unsafe { mem::transmute(e.source_agent()) },
                _                             => {},
            }
            */
        }
/*
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
                died:          None,
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

            match e.state_change() {
                CombatStateChange::ChangeDead => meta.died = Some(e.time()),
                CombatStateChange::LogStart   => start = e.value_as_time(),
                CombatStateChange::LogEnd     => end   = e.value_as_time(),
                // CombatStateChange::Language   => lang  = Language::from_agent_id(e.source_agent()),
                // FIXME: Do not use transmute, use the proper event conversion
                CombatStateChange::GwBuild    => build = unsafe { mem::transmute(e.source_agent()) },
                CombatStateChange::ShardId    => shard = unsafe { mem::transmute(e.source_agent()) },
                _                             => {},
            }
        }
        */

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
        self.buffer.skills.iter().chain(UNLISTED_SKILLS.iter())
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

        for s in self.skills.iter().filter(|s| s.id != 0) {
            map.serialize_entry(&{s.id}, s.name())?;
        }

        map.end()
    }
}