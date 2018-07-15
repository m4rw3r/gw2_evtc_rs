
use types::AgentId;
use types::Profession;
use types::InstanceId;
use IntoEvent;
use types::EventType;
use types::SpeciesId;

use event;

use std::fmt;
use std::mem;
use std::slice;
use std::str;

/// Array of unlisted skills which are not part of the evtc-file
pub static UNLISTED_SKILLS: &'static [Skill] = &[
    Skill { id: 1066,  name: *b"Resurrect\0                                                      "},
    Skill { id: 1175,  name: *b"Bandage\0                                                        "},
    Skill { id: 65001, name: *b"Dodge\0                                                          "},
    // TODO: Add boss-specific skills
];

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum CombatDataVersion {
    V1,
    V2,
}

#[repr(packed)]
#[derive(Debug, Copy, Clone)]
pub struct Header {
    pub version:  [u8; 12],
    _pad1:        u8,
    pub boss_id:  SpeciesId,
    pub position: u8,
    pub agents:   u32,
}

impl Header {
    fn combat_data_version(&self) -> CombatDataVersion {
        if self.version[12] == 1 {
            CombatDataVersion::V2
        }
        else {
            CombatDataVersion::V1
        }
    }
}

#[repr(packed)]
#[derive(Copy, Clone)]
pub struct Agent {
    pub id:            AgentId,
    profession:        u32,
    pub is_elite:      u32,
    pub toughness:     u16,
    pub concentration: u16,
    pub healing:       u16,
    _pad2_1:           u16,
    pub condition_dmg:  u16,
    _pad2_2:           u16,
    // Character name [null] Account name [null] Subgroup string literal [null]
    name:            [u8; 68],
}

impl fmt::Debug for Agent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Agent({}: {} ({}) {}, is_elite: {}, toughness: {}, healing: {}, condition_dmg: {})",
            {self.id}, self.name(), self.account_name(), self.profession(), {self.is_elite}, {self.toughness}, {self.healing}, {self.condition_dmg})
    }
}

impl Agent {
    pub fn id(&self) -> AgentId {
        self.id
    }

    pub fn name(&self) -> &str {
        // All agent and skill names use UTF8 according to deltaconnected
        unsafe { str::from_utf8_unchecked(self.name.split(|&c| c == 0).next().expect("Invalid C-string in EVTC Actor data")) }
    }

    pub fn account_name(&self) -> &str {
        // All agent and skill names use UTF8 according to deltaconnected
        unsafe { str::from_utf8_unchecked(self.name.split(|&c| c == 0).nth(1).expect("Invalid C-string in EVTC Actor data")) }
    }

    pub fn subgroup(&self) -> &str {
        // All agent and skill names use UTF8 according to deltaconnected
        unsafe { str::from_utf8_unchecked(self.name.split(|&c| c == 0).nth(2).expect("Invalid C-string in EVTC Actor data")) }
    }

    pub fn profession(&self) -> Profession {
        match (self.is_elite, self.profession) {
            (0xFFFFFFFF, x)  => if x & 0xffff0000 == 0xffff0000 { Profession::Gadget } else { Profession::NonPlayableCharacter }
            (0, 1)           => Profession::Guardian,
            (0, 2)           => Profession::Warrior,
            (0, 3)           => Profession::Engineer,
            (0, 4)           => Profession::Ranger,
            (0, 5)           => Profession::Thief,
            (0, 6)           => Profession::Elementalist,
            (0, 7)           => Profession::Mesmer,
            (0, 8)           => Profession::Necromancer,
            (0, 9)           => Profession::Revenant,
            (27, _) | (1, 1) => Profession::Dragonhunter,
            (18, _) | (1, 2) => Profession::Berserker,
            (43, _) | (1, 3) => Profession::Scrapper,
            (5,  _) | (1, 4) => Profession::Druid,
            (7,  _) | (1, 5) => Profession::Daredevil,
            (48, _) | (1, 6) => Profession::Tempest,
            (40, _) | (1, 7) => Profession::Chronomancer,
            (34, _) | (1, 8) => Profession::Reaper,
            (52, _) | (1, 9) => Profession::Herald,
            (55, _)          => Profession::Soulbeast,
            (56, _)          => Profession::Weaver,
            (57, _)          => Profession::Holosmith,
            (58, _)          => Profession::Deadeye,
            (59, _)          => Profession::Mirage,
            (60, _)          => Profession::Scourge,
            (61, _)          => Profession::Spellbreaker,
            (62, _)          => Profession::Firebrand,
            (63, _)          => Profession::Renegade,
            _                => Profession::Unknown,
        }
    }

    /// If the actor is a non-playable-character (NPC) then this method will return
    /// its species id.
    pub fn species_id(&self) -> Option<SpeciesId> {
        match (self.is_elite, self.profession & 0xffff0000) {
            (0xffffffff, 0xffff0000) => None,
            (0xffffffff, _)          => Some(unsafe { SpeciesId::new((self.profession & 0xffff) as u16) }),
            _                        => None,
        }
    }
}

#[repr(packed)]
#[derive(Copy, Clone)]
pub struct Skill {
    pub id: u32,
    name:   [u8; 64],
}

impl fmt::Debug for Skill {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Skill({}, {})", {self.id}, self.name())
    }
}

impl Skill {
    pub fn name(&self) -> &str {
        unsafe { str::from_utf8_unchecked(self.name.split(|&c| c == 0).next().expect("Invalid C-string in EVTC Skill data")) }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub enum Language {
    English = 0,
    French  = 2,
    German  = 3,
    Spanish = 4,
}

impl Language {
    pub fn from_agent_id(id: u64) -> Language {
        match id {
            2 => Language::French,
            3 => Language::German,
            4 => Language::Spanish,
            _ => Language::English,
        }
    }
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IFF {
    Friend  = 0,
    Foe     = 1,
    Unknown = 2,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HitResult {
    // good physical hit
    Normal      = 0, 
    // physical hit was crit
    Crit        = 1, 
    // physical hit was glance
    Glance      = 2, 
    // physical hit was blocked eg. mesmer shield 4
    Block       = 3, 
    // physical hit was evaded, eg. dodge or mesmer sword 2
    Evade       = 4, 
    // physical hit interrupted something
    Interrupt   = 5, 
    // physical hit was "invlun" or absorbed eg. guardian elite
    Absorb      = 6, 
    // physical hit missed
    Blind       = 7, 
    // physical hit was killing hit
    KillingBlow = 8, 
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CombatActivation {
    // Not used - not this kind of event
    None       = 0,
    // Without quickness
    Normal     = 1,
    // With quickness (+50% animation-speed)
    Quickness  = 2,
    // Cancel with reaching channel time
    CancelFire = 3,
    // Cancel without reaching channel time
    Cancel     = 4,
    // Animation completed fully
    Reset      = 5,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CombatStateChange {
    // not used - not this kind of event
    None            = 0,
    // src_agent entered combat, dst_agent is subgroup
    EnterCombat     = 1,
    // src_agent left combat
    ExitCombat      = 2,
    // src_agent is now alive
    ChangeUp        = 3,
    // src_agent is now dead
    ChangeDead      = 4,
    // src_agent is now downed
    ChangeDown      = 5,
    // src_agent is now in game tracking range
    Spawn           = 6,
    // src_agent is no longer being tracked
    Despawn         = 7,
    // src_agent has reached a health marker. dst_agent = percent * 10000 (eg. 99.5% will be 9950)
    HealthUpdate    = 8,
    // log start. value = server unix timestamp **uint32**. buff_dmg = local unix timestamp. src_agent = 0x637261 (arcdps id)
    LogStart        = 9,
    // log end. value = server unix timestamp **uint32**. buff_dmg = local unix timestamp. src_agent = 0x637261 (arcdps id)
    LogEnd          = 10,
    // src_agent swapped weapon set. dst_agent = current set id (0/1 water, 4/5 land)
    WeapSwap        = 11,
    // src_agent has had it's maximum health changed. dst_agent = new max health
    MaxHealthUpdate = 12,
    // src_agent will be agent of "recording" player
    PointOfView     = 13,
    // src_agent will be text language
    Language        = 14,
    // src_agent will be game build
    GwBuild         = 15,
    // src_agent will be sever shard id
    ShardId         = 16,
    // src_agent is self, dst_agent is reward id, value is reward type. these are the wiggly boxes that you get
    Reward          = 17,
    // combat event that will appear once per buff per agent on logging start (zero duration, buff==18)
    BuffInitial     = 18,
    // src_agent changed, cast float* p = (float*)&dst_agent, access as x/y/z (float[3])
    Position        = 19,
    // src_agent changed, cast float* v = (float*)&dst_agent, access as x/y/z (float[3])
    Velocity        = 20,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CombatBuffRemove {
    // Not used - not this kind of event
    None   = 0,
    // All buff stacks removed
    All    = 1,
    // Single stack removed. Disabled on server trigger, will happen for each stack on cleanse
    Single = 2,
    // Autoremoved by outofcombat or allstack (ignore for strip/cleanse calc, use for in/out volume)
    Manual = 3,
}

// TODO: Implement all public stuff as a trait
#[repr(packed)]
#[derive(Debug, Clone, Copy)]
pub struct CombatEvent {
    // timegettime() at time of event
    time:              u64,
    // Unique identifier
    src_agent:         u64,
    // Unique identifier
    dst_agent:         u64,
    // Event-specific
    value:             i32,
    // Estimated buff damage. Zero on application event
    buff_dmg:          i32,
    // Estimated overwritten stack duration for buff application
    overstack:         u16,
    // Skill ID
    skill_id:          u16,
    // Agent map instance id
    src_instid:        u16,
    // Agent map instance id
    dst_instid:        u16,
    // Master source agent map instance id if source is a minion/pet
    src_master_instid: u16,
    _pad8:             u64,
    _pad1:             u8,
    iff:               IFF,
    // Buff application, removal, or damage event
    buff:              u8,
    result:            HitResult,
    is_activation:     CombatActivation,
    // buff removed. src=relevant, dst=caused it (for strips/cleanses). from cbtr enum
    is_buffremove:     CombatBuffRemove,
    // source agent health was over 90%
    is_src_ninety:     u8,
    // target agent health was under 50%
    is_dst_fifty:      u8,
    // source agent was moving
    is_src_moving:     u8,
    is_statechange:    CombatStateChange,
    // If source was flanking target
    is_flanking:       u8,
    // All or part damage was vs barrier/shield
    is_shields:        u8,
    _pad2:             u16,
}

impl CombatEvent {
    fn src_agent(&self)         -> AgentId { AgentId::new(self.src_agent) }
    fn dst_agent(&self)         -> AgentId { AgentId::new(self.dst_agent) }
    fn src_instid(&self)        -> InstanceId { InstanceId::new(self.src_instid) }
    fn dst_instid(&self)        -> InstanceId { InstanceId::new(self.dst_instid) }
    fn src_master_instid(&self) -> Option<InstanceId> {
        if self.src_master_instid == 0 {
            None
        }
        else {
            Some(InstanceId::new(self.src_master_instid))
        }
    }

    #[inline(always)]
    fn match_activation(&self) -> event::EventType {
        event::EventType::Agent {
            agent:           if self.is_activation == CombatActivation::None && self.is_buffremove != CombatBuffRemove::None { self.dst_agent()  } else { self.src_agent() },
            instance:        if self.is_activation == CombatActivation::None && self.is_buffremove != CombatBuffRemove::None { self.dst_instid() } else { self.src_instid() },
            master_instance: self.src_master_instid(),
            event:           match self.is_activation {
                CombatActivation::Normal => event::AgentEvent::Activation { skill: self.skill_id, cast: event::Activation::Normal(self.value as u32) },
                CombatActivation::Quickness => event::AgentEvent::Activation { skill: self.skill_id, cast: event::Activation::Normal(self.value as u32) },
                CombatActivation::CancelFire => event::AgentEvent::Activation { skill: self.skill_id, cast: event::Activation::CancelFire(self.value as u32) },
                CombatActivation::Cancel => event::AgentEvent::Activation { skill: self.skill_id, cast: event::Activation::Cancel(self.value as u32) },
                CombatActivation::Reset => event::AgentEvent::Activation { skill: self.skill_id, cast: event::Activation::Cancel(self.value as u32) },
                // TODO: Add cause
                CombatActivation::None => event::AgentEvent::WithTarget {
                    agent:    if self.is_buffremove != CombatBuffRemove::None { self.src_agent()  } else { self.dst_agent() },
                    instance: if self.is_buffremove != CombatBuffRemove::None { self.src_instid() } else { self.dst_instid() },
                    event: match self.is_buffremove {
                        CombatBuffRemove::All | CombatBuffRemove::Manual => event::TargetEvent::Buff(self.skill_id, event::Buff::RemoveAll),
                        CombatBuffRemove::Single                         => event::TargetEvent::Buff(self.skill_id, event::Buff::RemoveSingle),
                        CombatBuffRemove::None                           => match (self.buff > 0, self.buff_dmg == 0) {
                            (true, true) => 
                                // TODO: Add info about duration and number of stacks and so on
                                event::TargetEvent::Buff(self.skill_id, event::Buff::Application),
                            (x, _) => event::TargetEvent::Damage {
                                skill:  self.skill_id,
                                damage: if x { self.buff_dmg as i64 } else { self.value as i64 },
                                flanking: self.is_flanking > 0,
                                moving:   self.is_src_moving > 0,
                                src_over90: self.is_src_ninety > 0,
                                hit_type:  if x { event::HitType::Condi } else {
                                    match self.result {
                                        HitResult::Normal      => event::HitType::Normal, 
                                        HitResult::Crit        => event::HitType::Crit, 
                                        HitResult::Glance      => event::HitType::Glance, 
                                        HitResult::Block       => event::HitType::Block, 
                                        HitResult::Evade       => event::HitType::Evade, 
                                        HitResult::Interrupt   => event::HitType::Interrupt, 
                                        HitResult::Absorb      => event::HitType::Absorb, 
                                        HitResult::Blind       => event::HitType::Blind, 
                                        HitResult::KillingBlow => event::HitType::KillingBlow, 
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

impl IntoEvent for CombatEvent {
    #[inline]
    fn to_event(&self) -> event::Event {
        event::Event { time: self.time, event: match self.is_statechange {
            // src_agent will be text language
            CombatStateChange::Language        => event::EventType::Language(Language::from_agent_id(self.src_agent)),
            // src_agent will be game build
            CombatStateChange::GwBuild         => event::EventType::Gw2Build(self.src_agent),
            // src_agent will be sever shard id
            CombatStateChange::ShardId         => event::EventType::Gw2Build(self.src_agent),
            // log start. value = server unix timestamp **uint32**. buff_dmg = local unix timestamp. src_agent = 0x637261 (arcdps id)
            CombatStateChange::LogStart        =>
                event::EventType::LogStart { server: self.value as u32, local: self.buff_dmg as u32, arcdps_id: self.src_agent },
            // log end. value = server unix timestamp **uint32**. buff_dmg = local unix timestamp. src_agent = 0x637261 (arcdps id)
            CombatStateChange::LogEnd          =>
                event::EventType::LogEnd { server: self.value as u32, local: self.buff_dmg as u32, arcdps_id: self.src_agent },
            // src_agent entered combat, dst_agent is subgroup
            CombatStateChange::EnterCombat     =>
                event::EventType::Agent { agent: self.src_agent(), instance: self.src_instid(), master_instance: self.src_master_instid(), event: event::AgentEvent::EnterCombat(self.dst_agent) },
            // src_agent left combat
            CombatStateChange::ExitCombat      =>
                event::EventType::Agent { agent: self.src_agent(), instance: self.src_instid(), master_instance: self.src_master_instid(), event: event::AgentEvent::ExitCombat },
            // src_agent is now alive
            CombatStateChange::ChangeUp        =>
                event::EventType::Agent { agent: self.src_agent(), instance: self.src_instid(), master_instance: self.src_master_instid(), event: event::AgentEvent::ChangeUp },
            // src_agent is now dead
            CombatStateChange::ChangeDead      =>
                event::EventType::Agent { agent: self.src_agent(), instance: self.src_instid(), master_instance: self.src_master_instid(), event: event::AgentEvent::ChangeDead },
            // src_agent is now downed
            CombatStateChange::ChangeDown      =>
                event::EventType::Agent { agent: self.src_agent(), instance: self.src_instid(), master_instance: self.src_master_instid(), event: event::AgentEvent::ChangeDown },
            // src_agent is now in game tracking range
            CombatStateChange::Spawn           =>
                event::EventType::Agent { agent: self.src_agent(), instance: self.src_instid(), master_instance: self.src_master_instid(), event: event::AgentEvent::Spawn },
            // src_agent is no longer being tracked
            CombatStateChange::Despawn         =>
                event::EventType::Agent { agent: self.src_agent(), instance: self.src_instid(), master_instance: self.src_master_instid(), event: event::AgentEvent::Despawn },
            // src_agent has reached a health marker. dst_agent = percent * 10000 (eg. 99.5% will be 9950)
            CombatStateChange::HealthUpdate    =>
                event::EventType::Agent { agent: self.src_agent(), instance: self.src_instid(), master_instance: self.src_master_instid(), event: event::AgentEvent::HealthUpdate(self.dst_agent) },
            // src_agent swapped weapon set. dst_agent = current set id (0/1 water, 4/5 land)
            CombatStateChange::WeapSwap        =>
                event::EventType::Agent { agent: self.src_agent(), instance: self.src_instid(), master_instance: self.src_master_instid(), event: event::AgentEvent::WeaponSwap },
            // src_agent has had it's maximum health changed. dst_agent = new max health
            CombatStateChange::MaxHealthUpdate =>
                event::EventType::Agent { agent: self.src_agent(), instance: self.src_instid(), master_instance: self.src_master_instid(), event: event::AgentEvent::MaxHealthUpdate(self.dst_agent) },
            // src_agent will be agent of "recording" player
            CombatStateChange::PointOfView     =>
                event::EventType::Agent { agent: self.src_agent(), instance: self.src_instid(), master_instance: self.src_master_instid(), event: event::AgentEvent::PointOfView },
            // src_agent is self, dst_agent is reward id, value is reward type. these are the wiggly boxes that you get
            CombatStateChange::Reward          =>
                event::EventType::Agent { agent: self.src_agent(), instance: self.src_instid(), master_instance: self.src_master_instid(), event: event::AgentEvent::Reward(self.dst_agent, self.value as u32) },
            // combat event that will appear once per buff per agent on logging start (zero duration, buff==18)
            CombatStateChange::BuffInitial     => {
                // println!("BuffInitial: {:?}", self);

                event::EventType::Agent { agent: self.src_agent(), instance: self.src_instid(), master_instance: self.src_master_instid(), event: event::AgentEvent::BuffInitial }
            },
            // src_agent changed, cast float* p = (float*)&dst_agent, access as x/y/z (float[3])
            CombatStateChange::Position        => {
                let pos: &[f32; 3] = unsafe { mem::transmute(&self.dst_agent) };

                event::EventType::Agent { agent: self.src_agent(), instance: self.src_instid(), master_instance: self.src_master_instid(), event: event::AgentEvent::Position { x: pos[0], y: pos[1], z: pos[2] } }
            },
            // src_agent changed, cast float* v = (float*)&dst_agent, access as x/y/z (float[3])
            CombatStateChange::Velocity        => {
                let pos: &[f32; 3] = unsafe { mem::transmute(&self.dst_agent) };

                event::EventType::Agent { agent: self.src_agent(), instance: self.src_instid(), master_instance: self.src_master_instid(), event: event::AgentEvent::Position { x: pos[0], y: pos[1], z: pos[2] } }
            },
            // not used - not this kind of event
            CombatStateChange::None            => self.match_activation()
        } }
    }

    #[inline]
    fn time(&self) -> u64 {
        self.time
    }

    #[inline]
    fn event_type(&self) -> EventType {
        if self.is_statechange != CombatStateChange::None {
            return EventType::StateChange;
        }

        if self.is_activation != CombatActivation::None {
            return EventType::Activation;
        }

        if self.is_buffremove != CombatBuffRemove::None {
            return EventType::BuffRemove;
        }
        
        if self.buff > 0 {
            return EventType::BuffApplication;
        }

        return EventType::PhysicalHit;
    }

    #[inline]
    fn source_agent(&self) -> AgentId {
        if self.event_type() == EventType::BuffRemove { self.dst_agent() } else { self.src_agent() }
    }

    #[inline]
    fn target_agent(&self) -> AgentId {
        if self.event_type() == EventType::BuffRemove { self.src_agent() } else { self.dst_agent() }
    }

    #[inline]
    fn target_instance(&self) -> InstanceId {
        // TODO: Also flipped?
        InstanceId::new(self.dst_instid)
    }

    #[inline]
    fn source_instance(&self) -> InstanceId {
        // TODO: Also flipped?
        InstanceId::new(self.src_instid)
    }

    #[inline]
    fn master_source_instance(&self) -> InstanceId {
        // TODO: Also flipped?
        InstanceId::new(self.src_master_instid)
    }

    #[inline]
    fn state_change(&self) -> CombatStateChange {
        self.is_statechange
    }

    #[inline]
    fn buff_damage(&self) -> i64 {
        self.buff_dmg as i64
    }

    #[inline]
    fn damage(&self) -> i64 {
        match self.event_type() {
            EventType::PhysicalHit     => self.value as i64,
            EventType::BuffApplication => self.buff_dmg as i64,
            _                          => 0,
        }
    }

    #[inline]
    fn hit_result(&self) -> HitResult {
        self.result
    }

    #[inline]
    fn is_source_flanking(&self) -> bool {
        self.is_flanking > 0
    }

    #[inline]
    fn is_source_moving(&self) -> bool {
        self.is_src_moving > 0
    }

    #[inline]
    fn is_source_over90(&self) -> bool {
        self.is_src_ninety > 0
    }

    #[inline]
    fn skill_id(&self) -> u16 {
        self.skill_id
    }

    /// Only valid if state_change() is LogStart or LogEnd
    #[inline]
    fn value_as_time(&self) -> u32 {
        unsafe { mem::transmute(self.value) }
    }

    /// Only valid if state_change() is LogStart or LogEnd
    #[inline]
    fn buffdmg_as_time(&self) -> u32 {
        unsafe { mem::transmute(self.buff_dmg) }
    }
}

#[derive(Debug)]
pub struct Evtc {
    pub header: Header,
    pub agents: Vec<Agent>,
    pub skills: Vec<Skill>,
    pub events: Vec<CombatEvent>,
}

#[derive(Debug)]
#[derive(Copy, Clone)]
pub struct EvtcBuf<'a> {
    pub header: &'a Header,
    pub agents: &'a [Agent],
    pub skills: &'a [Skill],
    pub events: &'a [CombatEvent],
}

fn transmute_single<T: Copy>(buf: &[u8]) -> Option<(&T, &[u8])> {
    if buf.len() < mem::size_of::<T>() {
        return None;
    }

    Some((unsafe { mem::transmute(buf.as_ptr()) }, &buf[mem::size_of::<T>()..]))
}

fn transmute_slice<T: Copy>(buf: &[u8], cnt: usize) -> Option<(&[T], &[u8])> {
    if buf.len() < mem::size_of::<T>() * cnt {
        return None;
    }

    Some((unsafe { slice::from_raw_parts(buf.as_ptr() as *const T, cnt) }, &buf[mem::size_of::<T>() * cnt..]))
}

pub fn transmute(buffer: &[u8]) -> EvtcBuf {
    let (header, buffer)     = transmute_single::<Header>(buffer).expect("EVTC-data is missing header");
    let (agents, buffer)     = transmute_slice(buffer, header.agents as usize).expect("EVTC-data too short, failed to read agents");
    let (num_skills, buffer) = transmute_single::<u32>(buffer).expect("EVTC-data too short, missing skill count");
    let (skills, buffer)     = transmute_slice(buffer, *num_skills as usize).expect("EVTC-data too short, failed to extract skills");
    let (events, _buffer)    = transmute_slice(buffer, buffer.len() / mem::size_of::<CombatEvent>()).expect("EVTC-data too short, failed to extract events");

    EvtcBuf {
        header,
        agents,
        skills,
        events,
    }
}