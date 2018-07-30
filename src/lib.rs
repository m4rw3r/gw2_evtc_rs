extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate fnv;

mod metadata;
mod iterator;

pub mod event;
pub mod statistics;
pub mod buff;

pub use event::*;

use serde::ser::Serialize;
use serde::ser::Serializer;

use std::i64;
use std::mem;
use std::fmt;
use std::u64;

pub use iterator::EventIteratorExt;

pub use metadata::Agent;
pub use metadata::Metadata;
pub use metadata::SkillList;

macro_rules! const_assert {
    ($($condition:expr),+ $(,)*) => {
        let _ = [(); 0 - !($($condition)&&+) as usize];
    };
    ($label:ident; $($rest:tt)+) => {
        #[allow(non_snake_case, dead_code)]
        fn $label() {
            const_assert!($($rest)+);
        }
    };
}

/// An id of an agent present in the encounter
#[derive(Clone, Copy, Eq, PartialEq, Debug, Hash)]
pub struct AgentId(u64);

const_assert!(AgentIdSize; mem::size_of::<AgentId>() == 8);

impl fmt::Display for AgentId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "#{}", {self.0})
    }
}

impl Default for AgentId {
    fn default() -> Self {
        AgentId::empty()
    }
}

impl AgentId {
    /// Creates an empty agent id.
    ///
    /// 0 is never used.
    #[inline(always)]
    pub fn empty() -> Self {
        AgentId(0)
    }

    /// Wraps a `u64` in the `AgentId` struct.
    pub fn new(id: u64) -> Self {
        AgentId(id)
    }
}

/// An id representing an instance of an agent present in the encounter.
#[derive(Clone, Copy, Eq, PartialEq, Debug, Hash, Serialize)]
pub struct InstanceId(u16);

const_assert!(InstanceIdSize; mem::size_of::<InstanceId>() == 2);

impl fmt::Display for InstanceId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{{{}}}", {self.0})
    }
}

impl Default for InstanceId {
    fn default() -> Self {
        InstanceId::empty()
    }
}

impl InstanceId {
    /// Creates an empty instance id.
    #[inline(always)]
    pub fn empty() -> Self {
        InstanceId(0)
    }

    /// Wraps a `u16` in an `InstanceId`.
    #[inline(always)]
    pub fn new(id: u16) -> Self {
        InstanceId(id)
    }
}

/// An id representing the type of enemy/gadget an agent is.
#[derive(Clone, Copy, Eq, PartialEq, Debug, Hash, Serialize)]
pub struct SpeciesId(u16);

const_assert!(SpeciesIdSize; mem::size_of::<SpeciesId>() == 2);

impl fmt::Display for SpeciesId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "&{}", {self.0})
    }
}

impl Default for SpeciesId {
    fn default() -> Self {
        SpeciesId::empty()
    }
}

impl SpeciesId {
    /// Creates a new empty species id.
    #[inline(always)]
    pub fn empty() -> Self {
        SpeciesId(0)
    }

    /// Wraps a `u16` in a `SpeciesId`.
    #[inline(always)]
    pub fn new(id: u16) -> Self {
        SpeciesId(id)
    }
}

/// The type of profession, includes NPCs and Gadgets
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize)]
pub enum Profession {
    Gadget,
    NonPlayableCharacter,
    // Base professions
    Guardian,
    Warrior,
    Revenant,
    Engineer,
    Ranger,
    Thief,
    Elementalist,
    Mesmer,
    Necromancer,
    // Heart of Thorns professions
    Dragonhunter,
    Berserker,
    Herald,
    Scrapper,
    Druid,
    Daredevil,
    Tempest,
    Chronomancer,
    Reaper,
    // Path of Fire professions
    Soulbeast,
    Weaver,
    Holosmith,
    Deadeye,
    Mirage,
    Scourge,
    Spellbreaker,
    Firebrand,
    Renegade,
    // Unknown
    /// Unknown profession, should not happen
    Unknown,
}

impl Profession {
    /// Returns the base-profession of any given profession.
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

/// Raid-bosses
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
    // TODO: Add golems
    Unknown,
}

impl Boss {
    /// Produces a `Boss` id from a `SpeciesId`.
    pub fn from_species_id(species: SpeciesId) -> Boss {
        match species.0 {
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

#[derive(Debug, Clone, Serialize)]
struct TimeEntry {
    /// Timestamp, rounded to seconds, in microseconds
    time:        u64,
    /// Health fraction, scaled 10000x
    health:      Option<u64>,
    /// Damage done during this second
    damage:      i64,
    /// Boss damage done during this second
    boss_dmg:    i64,
    /// If the player got downed this second
    downed:      bool,
    /// If the player got revided from downed state this second
    revived:     bool,
    /// If the player got revided from downed state this second
    dead:        bool,
    /// If the player swapped weapon this second
    weapon_swap: bool,
    // TODO: Add Skill-casts, add boons (FnvHashMap<u16, BuffSnapshot>)
}

impl TimeEntry {
    #[inline]
    fn with_time(t: u64) -> Self {
        TimeEntry {
            time:        t,
            health:      None,
            damage:      0,
            boss_dmg:    0,
            downed:      false,
            revived:     false,
            dead:        false,
            weapon_swap: false,
        }
    }

    #[inline]
    fn has_data(&self) -> bool {
           self.health.is_some()
        || self.damage > 0
        || self.boss_dmg > 0
        || self.downed
        || self.revived
        || self.dead
        || self.weapon_swap
    }
}

/// The time-series data for a player
#[derive(Debug, Clone)]
pub struct TimeSeries {
    series: Vec<TimeEntry>,
}

impl TimeSeries {
    #[inline]
    pub fn new(meta: &Metadata) -> Self {
        assert!(meta.log_end() > meta.log_start());

        TimeSeries {
            series: Vec::with_capacity(((meta.log_end() - meta.log_start()) / 1000) as usize),
        }
    }

    #[inline]
    pub fn parse_agent(meta: &Metadata, agent: &Agent) -> Self {
        let mut series = Self::new(meta);
        let agent_id = agent.id();
        let instance = agent.instance_id();

        series.parse(meta.encounter_events().filter_map(move |e| e.from_agent_or_gadgets(agent_id, instance)), meta);

        series
    }

    #[inline]
    pub fn parse<I: Iterator<Item=T>, T: Source>(&mut self, mut iter: I, meta: &Metadata) {
        let mut entry = if let Some(event) = iter.next() {
            let mut entry = TimeEntry::with_time(event.time() / 1000);

            self.parse_item(&mut entry, event, meta);

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

            self.parse_item(&mut entry, e, meta);
        }
    }

    #[inline]
    fn parse_item<T: Source>(&mut self, entry: &mut TimeEntry, event: T, meta: &Metadata) {
        if let Some(state) = event.state_change() {
            match state {
                StateChange::ChangeDown      => entry.downed      = true,
                StateChange::ChangeUp        => entry.revived     = true,
                // Got to check if it is a minion which died
                StateChange::ChangeDead if event.master_instance().is_none() => entry.dead        = true,
                StateChange::HealthUpdate(h) => entry.health      = Some(h),
                StateChange::WeaponSwap      => entry.weapon_swap = true,
                _ => {},
            }
        }

        if let Some(e) = event.into_damage() {
            entry.damage += e.damage();

            if let Some(b) = e.targeting_any_of(meta.bosses().map(|b| b.id())) {
                entry.boss_dmg += b.damage();
            }
        }
    }
}

impl Serialize for TimeSeries {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
      where S: Serializer {
        self.series.serialize(serializer)
    }
}