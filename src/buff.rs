use event::BuffRemoval;
use event::Buff;

use AgentId;

use std::mem;

pub type Quickness = Simulator<Duration<Queue, [u32; 5]>>;
pub type Might     = Simulator<Intensity<Replace, [u32; 25]>>;
pub type BasicBuff = Simulator<Duration<Replace, [u32; 1]>>;

pub trait Stack {
    /// Creates a new empty stack.
    fn new() -> Self;
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
pub struct Duration<T: StackType, U: Sized>(T, U);

#[derive(Debug, Clone)]
pub struct Intensity<T: StackType, U: Sized>(T, U);

macro_rules! impl_intensity {
    ($t:ident, $n:expr) => {
impl Stack for Intensity<$t, [u32; $n]> {
    fn new() -> Self {
        Intensity($t, [0; $n])
    }

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
impl Stack for Duration<$t, [u32; $n]> {
    fn new() -> Self {
        Duration($t, [0; $n])
    }

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
impl_duration!(Queue, 25);
impl_intensity!(Replace, 1);
impl_intensity!(Replace, 25);
impl_intensity!(Queue,   25);
impl_intensity!(Replace, 1500);

#[derive(Debug, Clone, Serialize)]
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

impl<T: Stack> Simulator<T> {
    #[inline]
    pub fn new(agent: AgentId) -> Self {
        Simulator {
            stack:     Stack::new(),
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

#[macro_export]
macro_rules! buff_table {
    (
        $struct:ident (
            $( $buff:ident $id:tt $kind:ident $type:ident $stacks:tt ),+
            $(,)*
        )
    ) => {
#[derive(Debug, Clone, Serialize)]
pub struct $struct {
    $(
        $buff: Simulator<$kind<$type, [u32; $stacks]>>
    ),*
}

impl $struct {
    pub fn new(agent_id: AgentId) -> Self {
        $struct {
            $(
                $buff: Simulator::new(agent_id)
            ),+
        }
    }

    pub fn update(&mut self, time: u64) {
        $(
            self.$buff.update(time);
        )+
    }

    pub fn add_event<E: Buff>(&mut self, e: E) {
        match e.skill() {
        $(
            $id => self.$buff.add_event(e.clone()),
        )+
            _ => {},
        }
    }
}
    }
}

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