extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate fnv;

mod event;
mod types;
mod metadata;
pub mod raw;
pub mod statistics;

use serde::ser::Serialize;
use serde::ser::Serializer;

use std::i64;
use std::u64;
use std::u32;
use std::mem;

use raw::CombatStateChange;
use raw::HitResult;

pub use types::Profession;
pub use types::Boss;
pub use types::InstanceId;
pub use types::SpeciesId;
pub use types::AgentId;
pub use types::EventType as ET;

pub use metadata::Agent;
pub use metadata::Metadata;
pub use metadata::SkillList;
pub use event::*;


pub trait IntoEvent {
    fn to_event(&self) -> Event;

    #[inline]
    fn time(&self) -> u64;

    #[inline]
    fn event_type(&self) -> ET;

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
    fn value_as_time(&self) -> u32;

    #[inline]
    fn buffdmg_as_time(&self) -> u32;

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
            _ => self.target_agent() == agent.id(),
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
            _ => self.source_agent() == agent.id(),
        }
    }

    fn from_agent_and_gadgets(&self, agent: &Agent) -> bool {
        match self.state_change() {
            CombatStateChange::LogStart
            | CombatStateChange::LogEnd
            | CombatStateChange::Language
            | CombatStateChange::ShardId
            | CombatStateChange::GwBuild => false,
            _ => self.source_agent() == agent.id() || self.master_source_instance() == agent.instance_id(),
        }
    }

    #[inline]
    fn is_boon(&self) -> bool {
        match self.event_type() {
            ET::BuffRemove      => true,
            ET::BuffApplication => self.buff_damage() == 0,
            _                          => false
        }
    }

    #[inline]
    fn is_physical_hit(&self) -> bool {
        self.event_type() == ET::PhysicalHit
    }
    
    #[inline]
    fn is_condition_tick(&self) -> bool {
        self.event_type() == ET::BuffApplication && self.buff_damage() > 0
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
        assert!(meta.log_end() > meta.log_start());

        TimeSeries {
            series: Vec::with_capacity(((meta.log_end() - meta.log_start()) / 1000) as usize),
        }
    }

    pub fn parse_agent(meta: &Metadata, agent: &Agent) -> Self {
        let mut series = Self::new(meta);

        series.parse(meta.encounter_events().filter(|e| e.from_agent_and_gadgets(agent)));

        series
    }

    #[inline]
    pub fn parse<'a, E: 'a + IntoEvent, I: Iterator<Item=&'a E>>(&mut self, mut iter: I) {
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

    fn parse_item<'a, E: 'a + IntoEvent>(&mut self, entry: &mut TimeEntry, event: &E) {
        match (event.state_change(), ) {
            (CombatStateChange::ChangeDown,)   => entry.downed      = true,
            (CombatStateChange::HealthUpdate,) => entry.health      = Some(unsafe { mem::transmute(event.target_agent()) }),
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