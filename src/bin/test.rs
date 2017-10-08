extern crate evtc;
extern crate specs;

use std::collections::HashMap;
use std::fs::File;
use std::mem;
use std::env;
use specs::{DispatcherBuilder, Join};

#[derive(Debug, Eq, PartialEq)]
struct Id(u64);

impl specs::Component for Id {
    type Storage = specs::VecStorage<Self>;
}

#[derive(Debug)]
struct Name(String);

impl specs::Component for Name {
    type Storage = specs::VecStorage<Self>;
}

#[derive(Debug)]
struct IncomingEvents(Vec<evtc::raw::CombatEvent>);

impl specs::Component for IncomingEvents {
    type Storage = specs::VecStorage<Self>;
}

#[derive(Debug)]
struct OutgoingEvents(Vec<evtc::raw::CombatEvent>);

impl specs::Component for OutgoingEvents {
    type Storage = specs::VecStorage<Self>;
}

#[derive(Debug, Copy, Clone)]
struct Time(u64);

impl specs::Component for Time {
    type Storage = specs::VecStorage<Self>;
}

#[derive(Debug, Copy, Clone)]
struct DeltaTime(u64);

impl specs::Component for DeltaTime {
    type Storage = specs::VecStorage<Self>;
}

#[derive(Debug, Copy, Clone)]
struct Quickness([u64; 5]);

impl specs::Component for Quickness {
    type Storage = specs::VecStorage<Self>;
}

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

struct QuicknessSystem;

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

struct Logger;

impl<'a> specs::System<'a> for Logger {
    type SystemData = (specs::Fetch<'a, Time>, specs::ReadStorage<'a, Name>, specs::ReadStorage<'a, Quickness>);

    fn run(&mut self, (time, name, quick): Self::SystemData) {
        for (n, q) in (&name, &quick).join() {
            if q.stacks() > 0 {
                println!("{}, {}, {}", time.0, n.0, q.stacks());
            }
        }
    }
}

fn create_dispatcher<'a, 'b>(evtc: &evtc::raw::Evtc) -> specs::Dispatcher<'a, 'b> {
    let mut builder = DispatcherBuilder::new()
        // Basic systems which operate independently of input
        .add(QuicknessSystem, "quickness", &[]);

    for s in &evtc.skills {
        // TODO: More buffs
        if s.name() == "Quickness" {
            builder = builder.add(QuicknessAdditionSystem(s.id as u16), "quickness_add", &["quickness"]);
        }
    }

    builder = builder.add(Logger, "logger", &["quickness", "quickness_add"]);

    builder.build()
}

fn main() {
    let file = File::open(env::args().nth(1).expect("missing argument to executable")).expect("could not open file");

    let evtc = evtc::raw::read(file).expect("Failed reading EVTC file");

    let mut world = specs::World::new();
    let mut inc_event_queue = HashMap::new();
    let mut out_event_queue = HashMap::new();

    world.register::<Id>();
    world.register::<Name>();
    world.register::<IncomingEvents>();
    world.register::<OutgoingEvents>();
    world.register::<Quickness>();

    let mut dispatcher = create_dispatcher(&evtc);

    for a in evtc.actors {
        let entity_id = world.create_entity()
            .with(Id(a.id))
            .with(Name(a.name()))
            .with(Quickness::default())
            // TODO: Should not be necessary, it should be initialized/removed?
            .build();

        println!("{:?}", a);

        out_event_queue.insert(a.id, (entity_id, Vec::new()));
        inc_event_queue.insert(a.id, (entity_id, Vec::new()));
    }

    /*let mut evtc_reader = EvtcReaderSystem;

    evtc_reader.run_now(&world.res);*/

    let mut time   = evtc.events.first().expect("EVTC log is empty").time;
    let mut events = evtc.events.iter();
    let mut ticks  = 0;
    let mut delta  = 0;

    while let Some(e) = events.next() {
        if e.time != time {
            world.write::<IncomingEvents>().clear();
            world.write::<OutgoingEvents>().clear();

            for &mut(id, ref mut v) in inc_event_queue.values_mut() {
                if v.is_empty() {
                    continue;
                }

                let w = mem::replace(v, Vec::new());

                world.write().insert(id, IncomingEvents(w));
            }

            for &mut(id, ref mut v) in out_event_queue.values_mut() {
                if v.is_empty() {
                    continue;
                }

                let w = mem::replace(v, Vec::new());

                world.write().insert(id, OutgoingEvents(w));
            }

            world.add_resource(Time(time));
            world.add_resource(DeltaTime(delta));

            dispatcher.dispatch(&world.res);
            world.maintain();

            ticks += 1;

            delta = e.time - time;
            time  = e.time;
        }

        if let Some(ref mut v) = inc_event_queue.get_mut(&e.dst_agent) {
            v.1.push(e.clone());
        }

        if let Some(ref mut v) = out_event_queue.get_mut(&e.src_agent) {
            v.1.push(e.clone());
        }
    }

    println!("Processed ticks {}", ticks);
}
