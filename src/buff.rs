use event::BuffRemoval;
use event::Buff;

use AgentId;

use serde::ser::Serialize;
use serde::ser::Serializer;
use serde::ser::SerializeMap;

use std::mem;
use std::fmt;

pub trait Stack: Default {
    /// The stack metadata
    const STACK_META: StackMeta;

    /// Pushes a new stack with the supplied duration in milliseconds, overstack is returned.
    fn push(&mut self, u32) -> u32;
    /// Updates the stack with the new timestamp difference, milliseconds.
    fn update(&mut self, u32);
    /// The total duration in milliseconds.
    fn sum(&self) -> u32;
    /// Clears the stack.
    fn clear(&mut self);
    /// The number of stacks present.
    fn stacks(&self) -> usize;
}

pub trait StackType {
    fn can_replace(usize, u32, u32) -> bool;
}

#[derive(Debug, Copy, Clone)]
pub struct Queue;

impl StackType for Queue {
    #[inline(always)]
    fn can_replace(index: usize, current: u32, new: u32) -> bool {
        current == 0 || (index > 0 && current < new)
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Replace;

impl StackType for Replace {
    #[inline(always)]
    fn can_replace(_: usize, current: u32, new: u32) -> bool {
        current < new
    }
}

/// Wrapper for a duration-stacking stack
///
/// # Tests
///
/// By casting Well of Action (8s) + shatters with Seize the Moment (3s) you max out on
/// 20s quickness. A reapplication of 6s of quickness using Signet of Inspiration will
/// add at most 3s seconds of quickness since it will replace a shatter-stack. The odd
/// interaction happens when you attempt to replace the almost-used Well of Action stack
/// when it is shorter. You still only get 3s until the stack has timed out completely.
///
/// This implies that the current stack is not replaced, even if it is smaller than the
/// new stack.
#[derive(Debug, Clone)]
pub struct Duration<T: StackType, U: Sized>(pub T, pub U);

#[derive(Debug, Clone)]
pub struct Intensity<T: StackType, U: Sized>(pub T, pub U);

macro_rules! impl_intensity {
    ($t:ident, $n:expr) => {
impl Default for Intensity<$t, [u32; $n]> {
    fn default() -> Self {
        Intensity($t, [0; $n])
    }
}

impl Stack for Intensity<$t, [u32; $n]> {
    const STACK_META: StackMeta = StackMeta::Intensity { max: $n };

    fn push(&mut self, mut stack: u32) -> u32 {
        for (i, s) in self.1.iter_mut().enumerate() {
            // Add if the stack is empty or if we are above the locked items and it is lower
            if $t::can_replace(i, *s, stack) {
                mem::swap(&mut stack, s);
            }

            if stack == 0 {
                return 0;
            }
        }

        // Return the overstack
        stack
    }

    fn update(&mut self, time: u32) {
        for d in self.1.iter_mut() {
            *d = d.saturating_sub(time);
        }
    }

    fn sum(&self) -> u32 {
        self.1.iter().filter(|&&i| i > 0).sum()
    }

    fn stacks(&self) -> usize {
        self.1.iter().filter(|&&i| i > 0).count()
    }

    fn clear(&mut self) {
        for d in self.1.iter_mut() {
            *d = 0;
        }
    }
}
    }
}

macro_rules! impl_duration /*_current_duration*/ {
    ($t:ident, $n:expr) => {
impl Default for Duration<$t, [u32; $n]> {
    fn default() -> Self {
        Duration($t, [0; $n])
    }
}

impl Stack for Duration<$t, [u32; $n]> {
    const STACK_META: StackMeta = StackMeta::Duration;

    fn push(&mut self, mut stack: u32) -> u32 {
        for (i, s) in self.1.iter_mut().enumerate() {
            // Add if the stack is empty or if we are above the locked items and it is lower
            if $t::can_replace(i, *s, stack) { // *s == 0 || (i >= $lock && *s < stack) {
                mem::swap(&mut stack, s);
            }

            if stack == 0 {
                return 0;
            }
        }

        // Return the overstack
        stack
    }

    fn update(&mut self, mut time: u32) {
        while time > 0 && self.1[0] > 0 {
            if self.1[0] > time {
                self.1[0] -= time;
                time       = 0;
            }
            else {
                time     -= self.1[0];
                self.1[0] = 0;
            }

            // Wait with the sort until we have a zero
            if self.1[0] == 0 {
                // Sort
                for i in 0..($n - 1) {
                    if self.1[i] < self.1[i + 1] {
                        self.1.swap(i, i + 1);
                    }
                }
            }
        }
    }

    fn sum(&self) -> u32 {
        self.1.iter().filter(|&&i| i > 0).sum()
    }

    fn stacks(&self) -> usize {
        for (i, &d) in self.1.iter().enumerate() {
            if d <= 0 {
                return i;
            }
        }

        return $n;
    }

    fn clear(&mut self) {
        for d in self.1.iter_mut() {
            *d = 0;
        }
    }
}
    }
}

impl_duration!(Replace, 1);
impl_duration!(Queue, 5);
impl_intensity!(Replace, 1);
impl_intensity!(Replace, 25);
impl_intensity!(Queue,   25);
impl_intensity!(Replace, 1500);

#[derive(Clone, Serialize)]
pub struct Simulator<T: Stack> {
    /// The boon-stack.
    #[serde(skip)]
    stack:     T,
    /// Last timestamp observed, milliseconds.
    #[serde(skip)]
    time:      u64,
    /// The agent we are interested in, needed since we have to check either source or target
    /// depending on if it is a cleanse or not.
    #[serde(skip)]
    agent:     AgentId,
    /// Sum of duration which got applied, milliseconds.
    uptime:    u32,
    /// Sum of extra duration which had no effect, milliseconds.
    overstack: u32,
    /// Sum of duration stripped, milliseconds.
    stripped:  u32,
}

impl<T: Stack> fmt::Debug for Simulator<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Simulator<{}> {{ stack: SKIPPED, time: {time:?}, agent: {agent:?}, uptime: {uptime:?}, overstack: {overstack:?}, stripped: {stripped:?} }}",
            stringify!(T),
            time=self.time,
            agent=self.agent,
            uptime=self.uptime,
            overstack=self.overstack,
            stripped=self.stripped
        )
    }
}

impl<T: Stack> Simulator<T> {
    #[inline]
    pub fn new(agent: AgentId, stack: T) -> Self {
        Simulator {
            stack:     stack,
            time:      0,
            uptime:    0,
            overstack: 0,
            stripped:  0,
            agent,
        }
    }

    #[inline]
    pub fn update(&mut self, time: u64) {
        self.stack.update(time.saturating_sub(self.time) as u32);

        self.time = time;
    }

    #[inline]
    pub fn add_event<E: Buff>(&mut self, e: E) {
        match e.removal() {
            // If we are not a buff-removal, the target receives the boon
            BuffRemoval::None   => if e.target_agent() == self.agent {
                let over = self.stack.push(e.duration());

                self.uptime    += e.duration().saturating_sub(over); // e.overstack());
                self.overstack += over; // e.overstack();
            },

            // If we are a buff-removal, it is the source which has the boon removed
            BuffRemoval::All | BuffRemoval::Single | BuffRemoval::Manual => if e.agent() == self.agent {
                let sum = self.stack.sum();

                self.uptime    = if sum > self.uptime { 0 } else { self.uptime - sum };
                self.stripped += sum;

                self.stack.clear();
            },
        }
    }

    /// Performs a final update and then subtracts the remaining stack duration from the uptime
    #[inline]
    pub fn finalize(&mut self, time: u64) {
        self.stack.update(time.saturating_sub(self.time) as u32);

        self.uptime = self.uptime.saturating_sub(self.stack.sum());
    }

    #[inline]
    pub fn stacks(&self) -> usize {
        self.stack.stacks()
    }

    #[inline]
    pub fn sum(&self) -> u32 {
        self.stack.sum()
    }

    #[inline]
    pub fn uptime(&self) -> u32 {
        self.uptime
    }

    #[inline]
    pub fn overstack(&self) -> u32 {
        self.overstack
    }

    #[inline]
    pub fn stripped(&self) -> u32 {
        self.stripped
    }
}

pub trait BoxedSimulator<E: Buff> {
    fn update(&mut self, time: u64);
    fn add_event(&mut self, e: E);
    fn stacks(&self) -> usize;
    fn sum(&self) -> u32;
    fn uptime(&self) -> u32;
    fn finalize(&mut self, u64);
    fn overstack(&self) -> u32;
    fn stripped(&self) -> u32;
}

impl<T: Stack, E:Buff> BoxedSimulator<E> for Simulator<T> {
    fn update(&mut self, time: u64) { Simulator::update(self, time) }
    fn add_event(&mut self, e: E) { Simulator::add_event(self, e) }
    fn stacks(&self) -> usize { Simulator::stacks(self) }
    fn sum(&self) -> u32 { Simulator::sum(self) }
    fn uptime(&self) -> u32 { Simulator::uptime(self) }
    fn finalize(&mut self, time: u64) { Simulator::finalize(self, time) }
    fn overstack(&self) -> u32 { Simulator::overstack(self) }
    fn stripped(&self) -> u32 { Simulator::stripped(self) }
}

impl<E:Buff> BoxedSimulator<E> for () {
    fn update(&mut self, _: u64) {}
    fn add_event(&mut self, _: E) {}
    fn stacks(&self) -> usize { 0 }
    fn sum(&self) -> u32 { 0 }
    fn uptime(&self) -> u32 { 0 }
    fn finalize(&mut self, _time: u64) {}
    fn overstack(&self) -> u32 { 0 }
    fn stripped(&self) -> u32 { 0 }
}

type BoxSimulator<E> = Box<BoxedSimulator<E> + Send>;

#[derive(Copy, Clone, Eq, PartialEq, Serialize, Debug)]
pub enum BuffType {
    Boon,
    Condition,
    Buff,
    Debuff,
    Item,
}

#[derive(Copy, Clone, Eq, PartialEq, Serialize, Debug)]
#[serde(tag = "type")]
pub enum StackMeta {
    Intensity {
        max: u16
    },
    Duration,
}

#[derive(Debug, Clone, Copy, Serialize)]
pub struct BuffMeta {
    pub name:       &'static str,
    pub stack: StackMeta,
    #[serde(rename="skillId")]
    pub skill_id:   u16,
    // TODO: Friendly/hostile, offensive/defensive and so on
}

#[derive(Debug, Clone, Copy)]
pub struct MetadataMap(&'static [BuffMeta]);

impl Serialize for MetadataMap {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer {
        let mut map = serializer.serialize_map(Some(self.0.len()))?;

        for v in self.0 {
            map.serialize_entry(&v.skill_id, &v)?;
        }

        map.end()
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Copy, Serialize)]
pub struct BuffSnapshot {
    stacks:    usize,
    sum:       u32,
    uptime:    u32,
    overstack: u32,
    stripped:  u32,
}

#[macro_export]
macro_rules! buff_table {
    (
        $visibility:tt $module:ident {
        $(
            $type:ty {
                $name:expr,
                skill_id: $id:expr,
                stack: $kind:ty
            }
        ),+
        $(,)*
        }
    ) => {

$visibility mod $module {
    use std::fmt;

    use fnv::FnvHashMap;

    use serde::ser::Serialize;
    use serde::ser::Serializer;
    use serde::ser::SerializeMap;

    use $crate::AgentId;
    use $crate::event::Buff;
    use $crate::buff::Simulator;
    use $crate::buff::BoxSimulator;
    use $crate::buff::BuffSnapshot;
    use $crate::buff::MetadataMap;
    use $crate::buff::BuffMeta;
    use $crate::buff::Stack;
    use $crate::buff::StackMeta;

    use super::*;

    const fn stack_meta<T: Stack>() -> StackMeta {
        T::STACK_META
    }

    pub static META_LIST: &'static [BuffMeta] = &[
        $(
        BuffMeta {
            name:     $name,
            stack:    stack_meta::<$kind>(),
            skill_id: $id,
        }
        ),*
    ];

    pub static META_MAP: MetadataMap = MetadataMap(&META_LIST);

    pub fn create_simulator<E: Buff>(agent_id: AgentId, skill_id: u16) -> BoxSimulator<E> {
        match skill_id {
            $(
            $id => Box::new(Simulator::<$kind>::new(agent_id, Default::default())),
            )*
            // FIXME: Need to skip the boon
            _ => Box::new(()),
        }
    }

    pub struct Map<E: Buff> {
        agent_id: AgentId,
        map:      FnvHashMap<u16, BoxSimulator<E>>,
    }

    impl<E: Buff> Map<E> {
        #[inline]
        pub fn new(agent_id: AgentId) -> Self {
            Map {
                map: FnvHashMap::default(),
                agent_id,
            }
        }

        pub fn update(&mut self, time: u64) {
            for v in self.map.values_mut() {
                v.update(time);
            }
        }

        pub fn add_event(&mut self, e: E) {
            let agent_id = self.agent_id;

            self.map.entry(e.skill())
                    .or_insert_with(|| create_simulator(agent_id, e.skill()))
                    .add_event(e);
        }

        pub fn snapshots<'a>(&'a self) -> impl Iterator<Item=(u16, BuffSnapshot)> + 'a {
            self.map.iter().filter(|(_, v)| v.uptime() > 0).map(|(&k, v)| (k, BuffSnapshot {
                stacks:    v.stacks(),
                sum:       v.sum(),
                uptime:    v.uptime(),
                overstack: v.overstack(),
                stripped:  v.stripped(),
            }))
        }

        #[inline]
        pub fn len(&self) -> usize {
            self.map.len()
        }

        pub fn finalize(&mut self, time: u64) {
            for b in self.map.values_mut() {
                b.finalize(time);
            }
        }
    }

    impl<E: Buff> fmt::Debug for Map<E> {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "Map {{ ")?;

            for (k, v) in self.snapshots() {
                write!(f, "{} => {:?}, ", k, v)?;
            }

            write!(f, "}}")
        }
    }

    impl<E: Buff> Serialize for Map<E> {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
          where S: Serializer {
            let mut map = serializer.serialize_map(Some(self.map.len()))?;

            for (k, v) in self.snapshots() {
                map.serialize_entry(&k, &v)?;
            }

            map.end()
        }
    }
}
    }
}

buff_table!(
pub table {
    BuffType::Boon{"Aegis",        skill_id: 743,   stack: Duration<Queue, [u32; 5]>},
    BuffType::Boon{"Alacrity",     skill_id: 30328, stack: Duration<Queue, [u32; 5]>},
    BuffType::Boon{"Fury",         skill_id: 725,   stack: Duration<Queue, [u32; 5]>},
    BuffType::Boon{"Might",        skill_id: 740,   stack: Intensity<Replace, [u32; 25]>},
    BuffType::Boon{"Protection",   skill_id: 717,   stack: Duration<Queue, [u32; 5]>},
    BuffType::Boon{"Quickness",    skill_id: 1187,  stack: Duration<Queue, [u32; 5]>},
    BuffType::Boon{"Regeneration", skill_id: 718,   stack: Duration<Queue, [u32; 5]>},
    BuffType::Boon{"Resistance",   skill_id: 26980, stack: Duration<Queue, [u32; 5]>},
    BuffType::Boon{"Retaliation",  skill_id: 873,   stack: Duration<Queue, [u32; 5]>},
    BuffType::Boon{"Stability",    skill_id: 1122,  stack: Intensity<Queue, [u32; 25]>},
    BuffType::Boon{"Swiftness",    skill_id: 719,   stack: Duration<Queue,  [u32; 5]>},
    BuffType::Boon{"Vigor",        skill_id: 726,   stack: Duration<Queue,  [u32; 5]>},
    // Conditions
    // Buffs
    BuffKind::Buff{"Banner of Strength",   skill_id: 14417, stack: Duration<Replace, [u32; 1]>},
    BuffKind::Buff{"Banner of Discipline", skill_id: 14449, stack: Duration<Replace, [u32; 1]>},
});

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn stack1() {
        let mut a = Duration(Queue, [3, 2, 0, 0, 0]);

        assert_eq!(a.stacks(), 2);
        assert_eq!(a.sum(), 5);

        a.push(4);

        // The currently active stack is not modified
        assert_eq!(a.1, [3, 4, 2, 0, 0]);
        assert_eq!(a.stacks(), 3);
        assert_eq!(a.sum(), 9);

        a.update(2);

        // Active stack is decreased
        assert_eq!(a.1, [1, 4, 2, 0, 0]);
        assert_eq!(a.stacks(), 3);
        assert_eq!(a.sum(), 7);

        a.update(4);

        assert_eq!(a.1, [1, 2, 0, 0, 0]);
        assert_eq!(a.stacks(), 2);
        assert_eq!(a.sum(), 3);
    }
}