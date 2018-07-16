use TargetEvent;
use HitType;
use EventType;
use Activation;
use Event;

use fnv::FnvHashMap;

use serde::ser::Serialize;
use serde::ser::Serializer;

use std::cmp;
use std::i64;

/// A sink for statistics
pub trait Sink<T>: Default {
    fn add_event(&mut self, e: T);
}

#[macro_export]
macro_rules! sink_from_iter {
    ($ty:ty, $u:ty) => {
impl ::std::iter::FromIterator<$u> for $ty {
    fn from_iter<I: IntoIterator<Item=$u>>(iter: I) -> Self {
        let mut s: $ty = Default::default();

        for e in iter {
            s.add_event(e);
        }

        s
    }
}
    }
}

sink_from_iter!(Hits, TargetEvent);
sink_from_iter!(Abilities, TargetEvent);
sink_from_iter!(ActivationLog, Event);

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

impl Sink<TargetEvent> for Hits {
    #[inline]
    fn add_event(&mut self, e: TargetEvent) {
        match e {
            TargetEvent::Damage {
                damage,
                flanking,
                moving,
                src_over90,
                hit_type,
                ..
            } => {
                debug_assert!(if hit_type.is_zero() { damage == 0 } else { true });

                self.total_damage += damage;
                self.hits         += 1;
                self.min_damage    = cmp::min(self.min_damage, damage);
                self.max_damage    = cmp::max(self.max_damage, damage);

                if flanking   { self.flanking += 1; }
                if moving     { self.moving   += 1; }
                if src_over90 { self.scholar  += 1; }

                match hit_type {
                    HitType::Crit      => self.criticals   += 1,
                    HitType::Glance    => self.glancing    += 1,
                    HitType::Block     => self.blocked     += 1,
                    HitType::Evade     => self.evaded      += 1,
                    HitType::Interrupt => self.interrupted += 1,
                    HitType::Absorb    => self.absorbed    += 1,
                    HitType::Blind     => self.missed      += 1,
                    _                  => {},
                }
            },
            _ => {},
        }
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

impl Sink<TargetEvent> for Abilities {
    #[inline]
    fn add_event(&mut self, e: TargetEvent) {
        match e {
            TargetEvent::Damage { skill, .. } => self.abilities.entry(skill).or_insert(Default::default()).add_event(e),
            _ => {}
        }
    }
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct ActivationEntry {
    time:      u64,
    skill:     u16,
    quickness: bool,
    canceled:  bool,
    duration:  u32,
}

#[derive(Clone, Debug, Default)]
pub struct ActivationLog {
    last: Option<(u64, u16, Activation)>,
    log:  Vec<ActivationEntry>,
}

impl Serialize for ActivationLog {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
      where S: Serializer {
        self.log.serialize(serializer)
    }
}

impl Sink<Event> for ActivationLog {
    #[inline]
    fn add_event(&mut self, e: Event) {
        if let Event { time, event: EventType::Activation { skill, cast }, .. } = e {
            match cast {
                Activation::Normal(_) | Activation::Quickness(_) => self.last = Some((time, skill, cast)),
                x => if let Some((time, s, start)) = self.last {
                    if skill != s {
                        return;
                    }

                    self.log.push(ActivationEntry {
                        time,
                        skill,
                        quickness: if let Activation::Quickness(_) = start { true } else { false },
                        canceled:  if let Activation::Cancel(_) = x { true } else { false },
                        duration: match x {
                            Activation::Normal(d)     => d,
                            Activation::Quickness(d)  => d,
                            Activation::CancelFire(d) => d,
                            Activation::Cancel(d)     => d,
                            Activation::Reset         => 0,
                        },
                    });

                    self.last = None;
                }
            }
        }
    }
}
