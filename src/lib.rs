extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate fnv;

pub mod event;
pub mod statistics;
//pub mod iterator;

mod metadata;
mod types;

//pub use iterator::EventIteratorExt;
pub use event::*;
pub use types::*;

//pub mod statistics;
//pub mod iterator;

use serde::ser::Serialize;
use serde::ser::Serializer;

use std::i64;
use std::u64;

//pub use iterator::EventIteratorExt;
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

/*
pub trait IntoEvent {
    #[inline]
    fn is_meta_event(&self) -> bool;

    #[inline]
    fn to_meta_event(&self) -> Option<MetaEvent>;

    #[inline]
    fn to_event(&self) -> Option<Event>;
}
*/

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