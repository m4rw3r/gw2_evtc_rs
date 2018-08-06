use event::Activation;
use event::CastType;
use event::Damage;
use event::Event;
use event::HitType;
use event::Source;
use event::StateChange;
use event::raw::WEAPON_SWAP;

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
    ($t:ident, $u:ident) => {
impl<T: $u> ::std::iter::FromIterator<T> for $t {
    fn from_iter<I: IntoIterator<Item=T>>(iter: I) -> Self {
        let mut s: $t = Default::default();

        for e in iter {
            s.add_event(e);
        }

        s
    }
}
    }
}

sink_from_iter!(Hits, Damage);
sink_from_iter!(Abilities, Damage);
sink_from_iter!(ActivationLog, Source);

#[derive(Debug, Copy, Clone)]
pub struct MinDamage(i64);

impl Default for MinDamage {
    fn default() -> Self {
        MinDamage(i64::MAX)
    }
}


impl MinDamage {
    fn add(&mut self, num: i64) {
        self.0 = cmp::min(num, self.0);
    }
}

impl Serialize for MinDamage {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
      where S: Serializer {
        if self.0 == i64::MAX {
            serializer.serialize_i64(0)
        }
        else {
            serializer.serialize_i64(self.0)
        }
    }
}

/// Statistics for hits
#[derive(Debug, Default, Copy, Clone, Serialize)]
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
    min_damage:    MinDamage,
    /// Maximum hit damage
    #[serde(rename="maxDamage")]
    max_damage:    i64,
}

impl<T: Damage> Sink<T> for Hits {
    #[inline]
    fn add_event(&mut self, e: T) {
        let hit_type = e.hit_type();
        let damage   = e.damage();

        debug_assert!(if hit_type.is_zero() { damage == 0 } else { true });

        self.hits += 1;

        if hit_type != HitType::Absorb {
            self.total_damage += damage;

            if damage != 0 {
                // TODO: Is this a good thing?
                self.min_damage.add(damage);
                self.max_damage = cmp::max(self.max_damage, damage);
            }
        }

        if e.flanking() { self.flanking += 1; }
        if e.moving()   { self.moving   += 1; }
        if e.over90()   { self.scholar  += 1; }

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

impl<T: Damage> Sink<T> for Abilities {
    #[inline]
    fn add_event(&mut self, e: T) {
        self.abilities.entry(e.skill()).or_insert(Default::default()).add_event(e)
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
    last: Option<(u64, u16, CastType)>,
    log:  Vec<ActivationEntry>,
}

impl Serialize for ActivationLog {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
      where S: Serializer {
        self.log.serialize(serializer)
    }
}

impl<T: Source> Sink<T> for ActivationLog {
    #[inline]
    fn add_event(&mut self, e: T) {
        match e.state_change() {
            // Weapon swaps can happen for gadgets/minions/pets too apparently, and sometimes they
            // are duplicated when using kits or conjures
            Some(StateChange::WeaponSwap) => if e.master_instance().is_none() &&
                self.log.last().map(|a| (a.time, a.skill)).unwrap_or((0, 0)) != (e.time(), WEAPON_SWAP) {
                self.log.push(ActivationEntry {
                    time:      e.time(),
                    skill:     WEAPON_SWAP,
                    quickness: false,
                    canceled:  false,
                    duration:  0,
                });
            },
            _ => {},
        }

        if let Some(e) = e.into_activation() {
            let cast  = e.cast();
            let skill = e.skill();

            match cast {
                CastType::Normal(_) | CastType::Quickness(_) => self.last = Some((e.time(), skill, cast)),
                x => if let Some((time, s, start)) = self.last {
                    if skill != s {
                        return;
                    }

                    self.log.push(ActivationEntry {
                        time,
                        skill,
                        quickness: if let CastType::Quickness(_) = start { true } else { false },
                        canceled:  if let CastType::Cancel(_)    = x { true } else { false },
                        duration: match x {
                            CastType::Normal(d)     => d,
                            CastType::Quickness(d)  => d,
                            CastType::CancelFire(d) => d,
                            CastType::Cancel(d)     => d,
                            CastType::Reset         => 0,
                        },
                    });

                    self.last = None;
                }
            }
        }
    }
}
