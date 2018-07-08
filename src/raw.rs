
use AgentId;
use Profession;
use InstanceId;
use Event;
use EventType;

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
    pub boss_id:  u16,
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
    pub condition:     u16,
    _pad2_2:           u16,
    // Character name [null] Account name [null] Subgroup string literal [null]
    name:            [u8; 68],
}

impl fmt::Debug for Agent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Agent({}: {} ({}) {}, is_elite: {}, toughness: {}, healing: {}, condition: {})",
            {self.id}, self.name(), self.account_name(), self.profession(), {self.is_elite}, {self.toughness}, {self.healing}, {self.condition})
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
    pub fn species_id(&self) -> Option<u16> {
        match (self.is_elite, self.profession & 0xffff0000) {
            (0xffffffff, 0xffff0000) => None,
            (0xffffffff, _)          => Some((self.profession & 0xffff) as u16),
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
    src_agent:         AgentId,
    // Unique identifier
    dst_agent:         AgentId,
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

impl Event for CombatEvent {
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
        if self.event_type() == EventType::BuffRemove { self.dst_agent } else { self.src_agent }
    }

    #[inline]
    fn target_agent(&self) -> AgentId {
        if self.event_type() == EventType::BuffRemove { self.src_agent } else { self.dst_agent }
    }

    #[inline]
    fn target_instance(&self) -> InstanceId {
        // TODO: Also flipped?
        InstanceId(self.dst_instid)
    }

    #[inline]
    fn source_instance(&self) -> InstanceId {
        // TODO: Also flipped?
        InstanceId(self.src_instid)
    }

    #[inline]
    fn master_source_instance(&self) -> InstanceId {
        // TODO: Also flipped?
        InstanceId(self.src_master_instid)
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