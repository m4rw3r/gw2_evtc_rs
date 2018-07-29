use event::BuffRemoval;
use event::Buff;

use AgentId;

use std::mem;

pub type Quickness = Simulator<Duration<Queue, [u32; 5]>>;

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

#[derive(Debug)]
pub struct Queue;

impl StackType for Queue {
    #[inline(always)]
    fn can_replace(index: usize, current: u32, new: u32) -> bool {
        current == 0 || (index > 0 && current < new)
    }
}

#[derive(Debug)]
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
#[derive(Debug)]
pub struct Duration<T: StackType, U: Sized>(T, U);

#[derive(Debug)]
pub struct Intensity<T: StackType, U: Sized>(T, U);

macro_rules! impl_intensity {
    ($t:ident, $n:expr) => {

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
                // TODO: Is this sorting actually what happens?
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
impl_intensity!(Replace, 25);
impl_intensity!(Replace, 1500);

#[derive(Debug)]
pub struct Simulator<T: Stack> {
    stack:     T,
    time:      u64,
    /// Sum of duration which got applied, milliseconds.
    uptime:    u32,
    /// Sum of extra duration which had no effect, milliseconds.
    overstack: u32,
    agent:     AgentId,
}

impl<T: Stack> Simulator<T> {
    #[inline]
    pub fn new(agent: AgentId) -> Self {
        Simulator {
            stack:     Stack::new(),
            time:      0,
            uptime:    0,
            overstack: 0,
            agent,
        }
    }

    #[inline]
    pub fn update(&mut self, time: u64) {
        assert!(time >= self.time, "time: {}, self.time: {}", time, self.time);

        self.stack.update((time - self.time) as u32);

        self.time = time;
    }

    #[inline]
    pub fn add_event<E: Buff>(&mut self, e: E) {
        match e.removal() {
            // If we are not a buff-removal, the target receives the boon
            BuffRemoval::None   => if e.target_agent() == self.agent {
                // self.overstack += cmp::max(0, e.overstack());

                let over = self.stack.push(e.duration());

                self.uptime    += e.duration().saturating_sub(over); // e.overstack());
                self.overstack += over; // e.overstack();

                println!("applied: {:>6}, overstacked: {:>6}, actual: {:>6}", e.duration(), e.overstack(), over);

            },

            // If we are a buff-removal, it is the source which has the boon removed
            BuffRemoval::All | BuffRemoval::Single | BuffRemoval::Manual => if e.agent() == self.agent {
                println!("Lost {} ms", self.stack.sum());

                self.uptime = if self.stack.sum() > self.uptime { 0 } else { self.uptime - self.stack.sum() };

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