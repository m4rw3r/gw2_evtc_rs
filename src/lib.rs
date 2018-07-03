use std::u64;
use std::collections::HashMap;

pub mod raw;

#[derive(Debug, Clone)]
struct AgentMetadata {
    instid:        u16,
    first_aware:   u64,
    last_aware:    u64,
    master_instid: u16,
    master_agent:  u64,
}

impl Default for AgentMetadata {
    fn default() -> Self {
        AgentMetadata {
            instid:        0,
            first_aware:   0,
            last_aware:    u64::MAX,
            master_instid: 0,
            master_agent:  0,
        }
    }
}

#[derive(Debug)]
pub struct Metadata {
    agent_data: HashMap<u64, AgentMetadata>,
}

impl Metadata {
    pub fn new(buffer: &raw::EvtcBuf) -> Self {
        let mut map = HashMap::<u64, AgentMetadata>::with_capacity(buffer.agents.len());

        for e in buffer.events.iter() {
            let master_agent = if e.src_master_instid != 0 {
                // TODO: Maybe check so our parent hasn't died yet? idk
                // FIXME: This does not seem to work properly
                map.iter().find(|(_id, m)| m.instid == e.src_master_instid /*&& m.first_aware < e.time*/).map(|(&id, _)| id)
            } else { None };

            let mut meta = map.entry(e.src_agent).or_insert(AgentMetadata {
                instid:        0,
                first_aware:   e.time,
                last_aware:    e.time,
                master_instid: 0,
                master_agent:  0,
            });

            // Apparently if it is not a combat-state-change then it is wrong
            if e.is_statechange != raw::CombatStateChange::None {
                meta.instid = e.src_instid;
            }

            meta.last_aware = e.time;

            if e.src_master_instid != 0 {
                meta.master_instid = e.src_master_instid;
                meta.master_agent  = master_agent.unwrap_or(meta.master_agent);
            }
        }

        for v in map.values().filter(|v| v.master_instid != 0 || v.master_agent != 0) {
            println!("{:?}", v);
        }

        Metadata {
            agent_data: map,
        }
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