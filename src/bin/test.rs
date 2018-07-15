#[macro_use]
extern crate evtc;
extern crate fnv;
extern crate memmap;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate zip;

use fnv::FnvHashMap;

use evtc::Agent;
use evtc::Boss;
use evtc::Event;
use evtc::raw::Language;
use evtc::statistics::Hits;
use evtc::statistics::Abilities;
use evtc::statistics::Sink;
use evtc::SkillList;
use evtc::TimeSeries;

use std::fs::File;
use std::env;

use zip::ZipArchive;

/// Separated hit-statistics depending on damage-type
#[derive(Debug, Clone, Default, Serialize)]
pub struct PowerCondiHits {
    power: Hits,
    condi: Hits,
}

// TODO: Should probably implement using derive
impl<E> Sink<E> for PowerCondiHits
  where E: Event {
    #[inline]
    fn add_event(&mut self, e: &E) {
        if e. is_physical_hit() {
            self.power.add_event(e);
        }

        if e.is_condition_tick() {
            self.condi.add_event(e);
        }
    }
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct AbilityAndTotal {
    total:     Hits,
    abilities: Abilities,
}

impl<E> Sink<E> for AbilityAndTotal
  where E: Event {
    #[inline]
    fn add_event(&mut self, e: &E) {
        self.total.add_event(e);
        self.abilities.add_event(e);
    }
}

// Should probably be a part of the derive
sink_from_iter!(PowerCondiHits);
sink_from_iter!(AbilityAndTotal);

#[derive(Debug, Clone, Serialize)]
struct AgentStatistics<'a> {
    agent:          &'a Agent,
    #[serde(rename="bossHits")]
    stats:          AbilityAndTotal,
}

#[derive(Debug, Clone, Serialize)]
struct PlayerSummary<'a> {
    agent:              &'a Agent,
    #[serde(rename="hits")]
    hit_stats:          PowerCondiHits,
    #[serde(rename="bossHits")]
    boss_hit_stats:     PowerCondiHits,
    agents:             Vec<AgentStatistics<'a>>,
    series:             TimeSeries,
}

#[derive(Debug, Clone, Serialize)]
struct BossSummary<'a> {
    agent:  &'a Agent,
    series: TimeSeries,
}

#[derive(Debug, Clone, Serialize)]
struct EncounterInfo {
    #[serde(rename="logStart")]
    log_start:    u32,
    #[serde(rename="logEnd")]
    log_end:      u32,
    #[serde(rename="gameBuild")]
    game_build:   u64,
    #[serde(rename="lang")]
    game_lang:    Language,
    #[serde(rename="serverShard")]
    server_shard: u64,
    #[serde(rename="logName")]
    log_name:     String,
    boss:         Boss,
    success:      bool,
}

#[derive(Debug, Clone, Serialize)]
struct Data<'a> {
    encounter: EncounterInfo,
    players:   Vec<PlayerSummary<'a>>,
    enemies:   Vec<BossSummary<'a>>,
    skills:    SkillList<'a>,
}

fn group_agents_by_species<'a, I: Iterator<Item=&'a Agent>>(iter: I) -> FnvHashMap<u16, Vec<&'a Agent>> {
    let mut map = FnvHashMap::default();

    for a in iter.filter(|a| a.species_id() != None) {
        map.entry(a.species_id().unwrap()).or_insert(Vec::new()).push(a);
    }

    map
}

fn parse_data(buffer: &[u8], logname: String) {
    let evtc = evtc::raw::transmute(buffer);
    let meta = evtc::Metadata::new(&evtc);

    let bosses: Vec<_> = meta.bosses().collect();

    let player_summaries = meta.agents().iter().filter(|a| a.is_player_character()).map(|a| PlayerSummary {
        agent: a,
        hit_stats:          meta.encounter_events().filter(|e| e.from_agent_and_gadgets(a)).collect(),
        boss_hit_stats:     meta.encounter_events().filter(|e| e.from_agent_and_gadgets(a) && bosses.iter().any(|b| e.targeting_agent(b))).collect(),
        agents:             (&[vec![a]]).iter().chain(group_agents_by_species(meta.agents_for_master(a)).values()).map(|minions| AgentStatistics {
            agent: minions[0],
            stats: meta.encounter_events().filter(|e| minions.iter().any(|m| e.from_agent(m)) && bosses.iter().any(|b| e.targeting_agent(b)) && e.is_damage()).collect(),
        }).collect(),
        series:    TimeSeries::parse_agent(&meta, a),
    }).collect();

    let boss_summaries: Vec<_> = meta.bosses().map(|b| BossSummary {
        agent: b,
        series: TimeSeries::parse_agent(&meta, b),
    }).collect();

    let data = Data {
        encounter: EncounterInfo {
            log_start:    meta.log_start(),
            log_end:      meta.log_end(),
            log_name:     logname,
            game_build:   meta.game_build(),
            game_lang:    meta.language(),
            server_shard: meta.server_shard(),
            boss:         meta.boss(),
            success:      meta.bosses().fold(true, |a, b| a && b.did_die()),
        },
        players:   player_summaries,
        enemies:   boss_summaries,
        skills:    meta.skill_list(),
    };

    println!("{}", serde_json::to_string_pretty(&data).unwrap());

        // println!("{} {}", a.name(), meta.encounter_events().filter(|e| e.from_agent_and_gadgets(a) && e.targeting_agent(boss)).map(|e| e.damage()).sum(): i64);

        // println!("{} {:?}", a.name(), HitStatistics::from_iterator(meta.encounter_events().filter(|e| e.from_agent_and_gadgets(a) && e.targeting_agent(boss) && e.is_physical_hit())));

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

        parse_data(&buffer[..], name);
    }
    else {
        let mmap = unsafe { memmap::Mmap::map(&file).expect("Failed to mmap() file") };

        parse_data(&mmap[..], name);
    }
}