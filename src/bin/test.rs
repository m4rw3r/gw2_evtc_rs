#![feature(type_ascription)]
extern crate evtc;
extern crate memmap;
extern crate rayon;
extern crate zip;

use std::fs::File;
use std::env;
use std::marker::PhantomData;
use std::ops::AddAssign;

use evtc::Event;
use evtc::EventType;
use evtc::HitStatistics;

use zip::ZipArchive;

trait Task: Default {
    fn parse_event(&mut self, time: u64, delta: u64, event: &evtc::raw::CombatEvent);

    fn parse_events<'a, I>(&mut self, mut events: I)
      where I: 'a,
            I: 'a + Iterator<Item=&'a evtc::raw::CombatEvent> {
        if let Some(first) = events.next() {
            let mut time   = first.time();
            let mut delta  = 0;

            self.parse_event(time, delta, first);

            for e in events {
                if e.time() != time {
                    delta = e.time() - time;
                    time  = e.time();
                }
                else {
                    delta = 0
                }

                self.parse_event(time, delta, e);
            }
        }
    }

    fn parse<'a, I>(events: I) -> Self
      where I: 'a,
            I: 'a + Iterator<Item=&'a evtc::raw::CombatEvent>,
            Self: Default {
        let mut data = Self::default();

        data.parse_events(events);

        data
    }
}

trait Property {
    type Type;

    fn get_data(event: &evtc::raw::CombatEvent) -> Self::Type;
}

/*
#[derive(Debug)]
struct SumValue(i64);

impl Default for SumValue {
    fn default() -> Self {
        SumValue(0)
    }
}

impl Task for SumValue {
    fn parse_event(&mut self, _time: u64, _delta: u64, event: &evtc::raw::CombatEvent) {
        self.0 += event.value();
    }
}

#[derive(Debug)]
struct Sum<P: Property> {
    pub value: P::Type,
    _type: PhantomData<P>,
}

impl<P, T> Default for Sum<P>
  where P: Property<Type=T>,
        T: AddAssign + Default {
    fn default() -> Self {
        Sum {
            value: Default::default(),
            _type: PhantomData,
        }
    }
}

impl<P, T> Task for Sum<P>
  where P: Property<Type=T>,
        T: AddAssign + Default {
    fn parse_event(&mut self, _time: u64, _delta: u64, event: &evtc::raw::CombatEvent) {
        self.value += P::get_data(event);
    }
}

struct Value;

impl Property for Value {
    type Type = i64;

    fn get_data(event: &evtc::raw::CombatEvent) -> Self::Type {
        event.value()
    }
}
*/

#[derive(Debug, Clone)]
struct AgentStatistics;

#[derive(Debug, Clone)]
struct PlayerSummary {
    total_damage:   i64,
    boss_damage:    i64,
    hit_stats:      HitStatistics,
    boss_hit_stats: HitStatistics,
    agents:         Vec<AgentStatistics>,
}

fn parse_data(buffer: &[u8]) {
    let evtc = evtc::raw::transmute(buffer);
    let meta = evtc::Metadata::new(&evtc);

    println!("Boss: {:?}, {}", meta.boss(), if meta.bosses().fold(true, |a, b| a && b.did_die()) { "success" } else { "failed" });

    let boss = meta.bosses().next().unwrap();

    for a in meta.agents().iter().filter(|a| a.is_player_character()) {
        // println!("{} {}", a.name(), meta.encounter_events().filter(|e| e.from_agent_and_gadgets(a) && e.targeting_agent(boss)).map(|e| e.damage()).sum(): i64);

        println!("{} {:?}", a.name(), HitStatistics::from_iterator(meta.encounter_events().filter(|e| e.from_agent_and_gadgets(a) && e.targeting_agent(boss))));
    }

/*
    println!("{:?}", meta);

    rayon::scope(|s| {
        for a in evtc.agents {
            s.spawn(move |_| {
                println!("{:?} {:?}", a.name(), Sum::<Value>::parse(evtc.events.iter().filter(|e| e.src_agent == a.id && ! e.is_buff() && e.value > 0)).value);

                /*let mut damage = SumValue::default();

                damage.parse_events(evtc.events.iter().filter(|e| e.src_agent == a.id && ! e.is_buff()));

                println!("{:?} {:?}", a.name(), damage.0);
                */
            })
        }
    })
    */
}

fn main() {
    let name       = env::args().nth(1).expect("missing argument to executable");
    let file       = File::open(&name).expect("could not open file");

    if name.ends_with(".zip") {
        use std::io::Read;

        let mut archive = ZipArchive::new(file).expect("Failed to unzip file");
        let mut file    = archive.by_index(0).expect("Failed to extract first file in archive");
        let mut buffer  = Vec::with_capacity(file.size() as usize);

        file.read_to_end(&mut buffer).expect("Failed to read first file in arcive");

        parse_data(&buffer[..]);
    }
    else {
        let mmap = unsafe { memmap::Mmap::map(&file).expect("Failed to mmap() file") };

        parse_data(&mmap[..]);
    }
}