use Event;

use raw::HitResult;

use fnv::FnvHashMap;

use serde::ser::Serialize;
use serde::ser::Serializer;

use std::cmp;
use std::i64;

/// A sink for statistics
pub trait Sink<E>: Default
  where E: Event {
    fn add_event(&mut self, e: &E);
}

#[macro_export]
macro_rules! sink_from_iter {
    ($ty:ty) => {
impl<'a, E> ::std::iter::FromIterator<&'a E> for $ty
  where E: 'a + Event {
    fn from_iter<I: IntoIterator<Item=&'a E>>(iter: I) -> Self {
        let mut s: $ty = Default::default();

        for e in iter {
            s.add_event(e);
        }

        s
    }
}
    }
}

sink_from_iter!(Hits);
sink_from_iter!(Abilities);

/// Statistics for hits
#[derive(Debug, Copy, Clone, Serialize)]
pub struct Hits {
    /// Total physical damage
    #[serde(rename="totalDamage")]
    total_damage:  i64,
    /// Total physical damage wasted being blocked, evaded, interrupted, absorbed (including invuln) or missed
    #[serde(rename="wastedDamage")]
    wasted_damage: i64,
    /// Total number of hits
    hits:          u32,
    /// Number of critical hits
    criticals:     u32,
    /// Number of hits which were done while source was flanking target
    flanking:      u32,
    /// Number of hits while source is over 90% HP
    scholar:       u32,
    /// Number of hits which were glancing hits
    glancing:      u32,
    /// Number of hits which were done while source was moving
    moving:        u32,
    /// Number of hits which were interrupted
    interrupted:   u32,
    /// Number of hits which got blocked by target
    blocked:       u32,
    /// Number of hits which got evaded by target
    evaded:        u32,
    /// Number of hits missed
    missed:        u32,
    /// Number of hits absorbed by target
    absorbed:      u32,
    /// Minimum hit damage
    #[serde(rename="minDamage")]
    min_damage:    i64,
    /// Maximum hit damage
    #[serde(rename="maxDamage")]
    max_damage:    i64,
}

impl Default for Hits {
    #[inline]
    fn default() -> Self {
        Self {
            total_damage:  0,
            wasted_damage: 0,
            hits:          0,
            criticals:     0,
            flanking:      0,
            scholar:       0,
            glancing:      0,
            moving:        0,
            interrupted:   0,
            blocked:       0,
            evaded:        0,
            missed:        0,
            absorbed:      0,
            min_damage:    i64::MAX,
            max_damage:    0,
        }
    }
}

impl<E> Sink<E> for Hits
  where E: Event {
    #[inline]
    fn add_event(&mut self, e: &E) {
        self.total_damage += match e.hit_result() {
              HitResult::Normal
            | HitResult::Crit
            | HitResult::Glance
            | HitResult::KillingBlow => e.damage(),
            _                        => 0,
        };
        self.wasted_damage += match e.hit_result() {
              HitResult::Block
            | HitResult::Evade
            | HitResult::Interrupt
            | HitResult::Absorb
            | HitResult::Blind => e.damage(),
            _                  => 0,
        };

        self.hits += 1;

        if e.is_source_flanking() { self.flanking += 1; }
        if e.is_source_moving()   { self.moving += 1; }
        if e.is_source_over90()   { self.scholar += 1; }

        if e.hit_result() == HitResult::Crit { self.criticals += 1; }
        if e.hit_result() == HitResult::Glance    { self.glancing += 1; }
        if e.hit_result() == HitResult::Interrupt { self.interrupted += 1; }
        if e.hit_result() == HitResult::Block     { self.blocked += 1; }
        if e.hit_result() == HitResult::Evade     { self.evaded += 1; }
        if e.hit_result() == HitResult::Blind     { self.missed += 1; }
        if e.hit_result() == HitResult::Absorb    { self.absorbed += 1; }

        self.min_damage = cmp::min(self.min_damage, e.damage());
        self.max_damage = cmp::max(self.max_damage, e.damage());
    }
}

#[derive(Debug, Clone)]
pub struct Abilities {
    abilities: FnvHashMap<u16, Hits>,
}

impl Serialize for Abilities {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
      where S: Serializer {
        self.abilities.serialize(serializer)
    }
}

impl Default for Abilities {
    #[inline]
    fn default() -> Self {
        Self {
            abilities: FnvHashMap::default(),
        }
    }
}

impl<E> Sink<E> for Abilities
  where E: Event {
    #[inline]
    fn add_event(&mut self, e: &E) {
        self.abilities.entry(e.skill_id()).or_insert(Default::default()).add_event(e);
    }
}