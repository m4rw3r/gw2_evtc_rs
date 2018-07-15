use fnv::FnvHashMap;

use evtc::raw;

use evtc::Agent;
use evtc::Boss;
use evtc::EventIteratorExt;
use evtc::HitType;
use evtc::Metadata;
use evtc::SkillList;
use evtc::SpeciesId;
use evtc::TargetEvent;
use evtc::TimeSeries;

use evtc::raw::Language;

use evtc::statistics::Abilities;
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
impl Sink<TargetEvent> for PowerCondiHits {
    #[inline]
    fn add_event(&mut self, e: TargetEvent) {
        match e {
            TargetEvent::Damage { hit_type: HitType::Condi, .. } => self.condi.add_event(e),
            TargetEvent::Damage { .. }                           => self.power.add_event(e),
            _ => {},
        }
    }
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct AbilityAndTotal {
    total:     Hits,
    abilities: Abilities,
}

impl Sink<TargetEvent> for AbilityAndTotal {
    #[inline]
    fn add_event(&mut self, e: TargetEvent) {
        self.total.add_event(e);
        self.abilities.add_event(e);
    }
}

// Should probably be a part of the derive
sink_from_iter!(PowerCondiHits, TargetEvent);
sink_from_iter!(AbilityAndTotal, TargetEvent);

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

fn group_agents_by_species<'a, I: Iterator<Item=&'a Agent>>(iter: I) -> FnvHashMap<SpeciesId, Vec<&'a Agent>> {
    let mut map = FnvHashMap::default();

    for a in iter.filter(|a| a.species_id() != None) {
        map.entry(a.species_id().unwrap()).or_insert(Vec::new()).push(a);
    }

    map
}

pub fn parse_data<W: Write>(buffer: &[u8], logname: String, writer: W) -> Result<(), JSONError> {
    let evtc = raw::transmute(buffer);
    let meta = Metadata::new(&evtc);

    let bosses: Vec<_> = meta.bosses().collect();

    let player_summaries = meta.agents().iter().filter(|a| a.is_player_character()).map(|a| PlayerSummary {
        agent: a,
        hit_stats:          meta.encounter_events().from_agent_and_gadgets(a).target_events().collect(),
        boss_hit_stats:     meta.encounter_events().from_agent_and_gadgets(a).targeting_any_of(&bosses[..]).target_events().collect(),
        agents:             (&[vec![a]]).iter().chain(group_agents_by_species(meta.agents_for_master(a)).values()).map(|minions| AgentStatistics {
            agent: minions[0],
            stats: meta.encounter_events().from_any_of(minions).targeting_any_of(&bosses[..]).target_events().collect(),
            // stats: meta.encounter_events().filter(|e| minions.iter().any(|m| e.from_agent(m)) && bosses.iter().any(|b| e.targeting_agent(b)) && e.is_damage()).collect(),
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

    serde_json::to_writer(writer, &data)
}