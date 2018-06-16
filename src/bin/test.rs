extern crate evtc;
extern crate memmap;
extern crate rayon;

use std::fs::File;
use std::env;

#[derive(Debug, Copy, Clone)]
struct Quickness([u64; 5]);

//impl specs::Component for Quickness {
//    type Storage = specs::VecStorage<Self>;
//}

impl Default for Quickness {
    fn default() -> Self {
        Quickness([0; 5])
    }
}

impl Quickness {
    fn decrease(&mut self, mut dtime: u64) {
        for i in (0..5).rev() {
            let d = dtime.saturating_sub(self.0[i]);

            self.0[i].saturating_sub(dtime);

            dtime = d;
        }

        self.0.sort();
    }

    fn increase(&mut self, time: u64) {
        for i in (0..5).rev() {
            if self.0[i] < time {
                self.0[i] = time;

                break;
            }
        }
    }

    fn stacks(&self) -> usize {
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

trait Task {
    fn parse(&mut self, time: u64, delta: u64, event: evtc::raw::CombatEvent);

    fn parse_events<I: Iterator<Item=evtc::raw::CombatEvent>>(&mut self, mut events: I) {
        if let Some(first) = events.next() {
            let mut time   = first.time;
            let mut delta  = 0;

            self.parse(time, delta, first);

            for e in events {
                if e.time != time {
                    delta = e.time - time;
                    time  = e.time;
                }
                else {
                    delta = 0
                }

                self.parse(time, delta, e);
            }
        }
    }
}

struct BuffApplications {
    agent:    u64,
    skill_id: u16,
    events:   Vec<evtc::raw::CombatEvent>,
}

#[derive(Debug)]
struct SumValue(i64);

impl Default for SumValue {
    fn default() -> Self {
        SumValue(0)
    }
}

impl Task for SumValue {
    fn parse(&mut self, _time: u64, _delta: u64, event: evtc::raw::CombatEvent) {
        self.0 += event.value as i64;
    }
}

fn main() {
    let file = File::open(env::args().nth(1).expect("missing argument to executable")).expect("could not open file");
    let mmap = unsafe { memmap::Mmap::map(&file).unwrap() };
    let evtc = evtc::raw::transmute(&mmap[..]);

    rayon::scope(|s| {
        for a in evtc.actors {
            s.spawn(move |_| {
                let mut damage = SumValue::default();

                damage.parse_events(evtc.events.iter().cloned().filter(|e| e.src_agent == a.id && ! e.is_buff()));

                println!("{:?} {:?}", a.name(), damage.0);
            })
        }
    })
}