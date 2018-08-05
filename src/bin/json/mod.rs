use fnv::FnvHashMap;

use evtc::Agent;
use evtc::AgentId;
use evtc::Boss;
use evtc::Damage;
use evtc::Event;
use evtc::EventIteratorExt;
use evtc::HitType;
use evtc::Language;
use evtc::Metadata;
use evtc::SkillList;
use evtc::SpeciesId;
use evtc::TimeSeries;
use evtc::timeseries::Series;
use evtc::timeseries::Entry;
use evtc::buff::MetadataMap;
use evtc::buff::BuffSnapshot;
use evtc::buff::table as buffs;
use evtc::event::Source;
use evtc::event::StateChange;
use evtc::event::raw::CombatEventV1;
use evtc::raw;
use evtc::statistics::Abilities;
use evtc::statistics::ActivationLog;
use evtc::statistics::Hits;
use evtc::statistics::Sink;

use serde_json;

use std::io::Write;

use serde_json::error::Error as JSONError;

/// Separated hit-statistics depending on damage-type
#[derive(Debug, Clone, Default, Serialize)]
pub struct PowerCondiHits {
    power: Hits,
    condi: Hits,
}

// TODO: Should probably implement using derive
impl<T: Damage> Sink<T> for PowerCondiHits {
    #[inline]
    fn add_event(&mut self, e: T) {
        match e.hit_type() {
            HitType::Condi => self.condi.add_event(e),
            _              => self.power.add_event(e),
        }
    }
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct AbilityAndTotal {
    total:     Hits,
    abilities: Abilities,
}

impl<T: Damage> Sink<T> for AbilityAndTotal {
    #[inline]
    fn add_event(&mut self, e: T) {
        self.total.add_event(e.clone());
        self.abilities.add_event(e);
    }
}

// Should probably be a part of the derive
sink_from_iter!(PowerCondiHits, Damage);
sink_from_iter!(AbilityAndTotal, Damage);

#[derive(Debug, Clone, Serialize)]
struct AgentStatistics<'a> {
    agent:     &'a Agent,
    #[serde(skip)]
    agent_ids: Vec<AgentId>,
    #[serde(rename="bossHits")]
    stats:     AbilityAndTotal,
}

impl<'a> AgentStatistics<'a> {
    fn new(agents: &Vec<&'a Agent>) -> Self {
        AgentStatistics {
            agent:     agents[0],
            agent_ids: agents.iter().map(|a| a.id()).collect(),
            stats:     Default::default(),
        }
    }
}

#[derive(Default, Serialize)]
struct TimeEntry {
    time:        u64,
    buffs:       Option<FnvHashMap<u16, BuffSnapshot>>,
    health:      Option<u64>,
    damage:      i64,
    #[serde(rename="bossDmg")]
    boss_dmg:    i64,
    downed:      bool,
    revived:     bool,
    dead:        bool,
    #[serde(rename="weaponSwap")]
    weapon_swap: bool,
}

impl Entry for TimeEntry {
    fn new(time: u64) -> Self {
        Self {
            time,
            ..Default::default()
        }
    }

    fn time(&self) -> u64 {
        self.time
    }
}

#[derive(Serialize)]
struct PlayerSummary<'a, E: Event> {
    agent:              &'a Agent,
    #[serde(rename="hits")]
    hit_stats:          PowerCondiHits,
    #[serde(rename="bossHits")]
    boss_hit_stats:     PowerCondiHits,
    agents:             Vec<AgentStatistics<'a>>,
    #[serde(rename="activationLog")]
    activation_log:     ActivationLog,
    buffs:              buffs::Map<E::BuffEvent>,
    #[serde(rename="incomingDamage")]
    incoming_damage:    AbilityAndTotal,
    series:             Series<TimeEntry>,
}

impl<'a, E: Source> PlayerSummary<'a, E> {
    fn new(meta: &'a Metadata<'a>, agent: &'a Agent) -> Self {
        let gadgets = group_agents_by_species(meta.agents_for_master(agent));

        PlayerSummary {
            agent,
            hit_stats:       Default::default(),
            boss_hit_stats:  Default::default(),
            agents:          (&[vec![agent]]).iter()
                                             .chain(gadgets.values())
                                             .map(AgentStatistics::new)
                                             .collect(),
            activation_log:  Default::default(),
            incoming_damage: Default::default(),
            buffs:           buffs::Map::new(agent.id()),
            series:          Series::new(meta),
        }
    }

    // TODO: Do we really filter events before this?
    fn parse<I: Iterator<Item=E>>(mut self, bosses: &[AgentId], i: I) -> Self {
        let mut t = 0;

        for event in i {
            // We only store entries per second
            let entry = self.series.current(event.time() / 1000);

            if event.time() != t {
                self.buffs.update(event.time());

                t = event.time();
            }

            // Snapshot buffs
            if entry.buffs.is_none() {
                entry.buffs = Some(self.buffs.snapshots().collect());
            }

            // Parse
            if let Some(state) = event.state_change() {
                match state {
                    StateChange::ChangeDown      => entry.downed      = true,
                    StateChange::ChangeUp        => entry.revived     = true,
                    // Got to check if it is a minion which died
                    StateChange::ChangeDead if event.master_instance().is_none() => entry.dead        = true,
                    StateChange::HealthUpdate(h) => entry.health      = Some(h),
                    StateChange::WeaponSwap      => entry.weapon_swap = true,
                    _ => {},
                }
            }

            if let Some(b) = event.clone().into_buff() {
                self.buffs.add_event(b);
            }

            if let Some(e) = event.clone()
                                  .targeting_agent(self.agent.id())
                                  .and_then(Event::into_damage) {
                self.incoming_damage.add_event(e);
            }

            if let Some(e) = event.clone()
                              .from_agent_or_gadgets(self.agent.id(), self.agent.instance_id()) {
                self.activation_log.add_event(e.clone());

                if let Some(d) = e.clone()
                                  .into_damage() {
                    self.hit_stats.add_event(d.clone());

                    entry.damage += d.damage();

                    if let Some(b) = d.clone()
                                      .targeting_any_of(bosses.iter().cloned()) {
                        self.boss_hit_stats.add_event(b.clone());

                        entry.boss_dmg += b.damage();

                        for a in &mut self.agents {
                            if a.agent_ids.contains(&e.agent()) {
                                a.stats.add_event(b.clone());
                            }
                        }
                    }
                }
            }
        }

        self.series.finalize();

        self
    }
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
    #[serde(rename="seriesStart")]
    series_start: u64,
    #[serde(rename="seriesEnd")]
    series_end:   u64,
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

#[derive(Serialize)]
struct Data<'a, E: Event> {
    encounter: EncounterInfo,
    players:   Vec<PlayerSummary<'a, E::SourceEvent>>,
    enemies:   Vec<BossSummary<'a>>,
    buffs:     MetadataMap,
    skills:    SkillList<'a>,
}

fn group_agents_by_species<'a, I: Iterator<Item=&'a Agent>>(iter: I) -> FnvHashMap<SpeciesId, Vec<&'a Agent>> {
    let mut map = FnvHashMap::default();

    for (i, a) in iter.filter_map(|a| a.profession().species_id().map(|i| (i, a))) {
        map.entry(i).or_insert(Vec::new()).push(a);
    }


    map
}

pub fn parse_data<W: Write>(buffer: &[u8], logname: String, pretty:bool, writer: W) -> Result<(), JSONError> {
    let evtc = raw::transmute(buffer);
    let meta = Metadata::new(&evtc);

    let bosses:  Vec<_> = meta.bosses().collect();
    let boss_ids: Vec<_> = bosses.iter().map(|a| a.id()).collect();

/*

    println!("Quickness:");
    println!("uptime:    {}", (quickness.uptime() as f64) / (meta.log_end() - meta.log_start()) as f64 / 1000.0);
    println!("overstack: {}", (quickness.overstack() as f64) / 1000.0);
    println!("Might:");
    println!("uptime:    {}", (might.uptime() as f64) / (meta.log_end() - meta.log_start()) as f64 / 1000.0);
    println!("overstack: {}", (might.overstack() as f64) / 1000.0);
    println!("Spotter:");
    println!("uptime:    {}", (spotter.uptime() as f64) / (meta.log_end() - meta.log_start()) as f64 / 1000.0);
    println!("overstack: {}", (spotter.overstack() as f64) / 1000.0);
    println!("fight:     {}", meta.log_end() - meta.log_start());
}

panic!("FOO");
*/
    let player_summaries = meta.agents()
                               .iter()
                               .filter(|a| a.profession().is_player_character())
                               .map(|a| PlayerSummary::new(&meta, a)
                                        // TODO: Is related to enough to get everything?
                                        .parse(&boss_ids[..], meta.encounter_events().related_to(a)))
                               .collect();

    let boss_summaries: Vec<_> = meta.bosses().map(|b| BossSummary {
        agent: b,
        series: TimeSeries::parse_agent(&meta, b),
    }).collect();

    let data: Data<&CombatEventV1> = Data {
        encounter: EncounterInfo {
            log_start:    meta.log_start_time(),
            log_end:      meta.log_end_time(),
            series_start: meta.log_start(),
            series_end:   meta.log_end(),
            log_name:     logname,
            game_build:   meta.game_build(),
            game_lang:    meta.language(),
            server_shard: meta.server_shard(),
            boss:         meta.boss(),
            success:      meta.bosses().fold(true, |a, b| a && b.did_die()),
        },
        players:   player_summaries,
        enemies:   boss_summaries,
        buffs:     buffs::META_MAP,
        skills:    meta.skill_list(),
    };

    if pretty {
        serde_json::to_writer_pretty(writer, &data)
    }
    else {
        serde_json::to_writer(writer, &data)
    }
}