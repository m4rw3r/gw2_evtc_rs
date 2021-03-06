use event::Activation;
use event::ActivationEvent;
use event::Buff;
use event::BuffEvent;
use event::CastType;
use event::Damage;
use event::DamageEvent;
use event::Event;
use event::HitType;
use event::Meta;
use event::MetaEvent;
use event::MetaEventData;
use event::Source;
use event::SourceEvent;
use event::StateChange;
use event::Target;
use event::TargetEvent;

use AgentId;
use InstanceId;
use Profession;
use SpeciesId;

use std::cmp;
use std::fmt;
use std::mem;
use std::slice;
use std::str;

/// Array of unlisted skills which are not part of the evtc-file
pub static UNLISTED_SKILLS: &'static [Skill] = &[
    Skill { id: 1066,  name: *b"Resurrect\0                                                      "},
    Skill { id: 1175,  name: *b"Bandage\0                                                        "},
    Skill { id: 65001, name: *b"Dodge\0                                                          "},
    Skill { id: WEAPON_SWAP as u32, name: *b"Weapon Swap\0                                                    "},
    // TODO: Add boss-specific skills
];

pub const WEAPON_SWAP: u16 = 65535 - 1;

/// The combat data version, `V1` is not binary-compatible with `V2`.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum CombatDataVersion {
    V1,
    V2,
}

/// EVTC-file header.
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
    /// The combat data version of the header.
    pub fn combat_data_version(&self) -> CombatDataVersion {
        // FIXME: This index seems to be wrong:
        //if self.version[12] == 1 {
        if self.version[11] == 1 {
            CombatDataVersion::V2
        }
        else {
            CombatDataVersion::V1
        }
    }
}

/// Stats for an agent.
#[repr(packed)]
#[derive(Copy, Clone)]
pub struct Agent {
    /// Agent id
    pub id:            AgentId,
    profession:        u32,
    is_elite:          u32,
    /// Relative amount of toughness of the agent.
    pub toughness:     u16,
    /// Relative amount of concentration of the agent.
    pub concentration: u16,
    /// Relative amount of healing power of the agent
    pub healing:       u16,
    _pad2_1:           u16,
    /// Relative amount of condition damage of the agent.
    pub condition_dmg: u16,
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
        unsafe { str::from_utf8_unchecked(self.name.split(|&c| c == 0).next().expect("Invalid C-string in EVTC Agent data")) }
    }

    pub fn account_name(&self) -> &str {
        // All agent and skill names use UTF8 according to deltaconnected
        unsafe { str::from_utf8_unchecked(self.name.split(|&c| c == 0).nth(1).expect("Invalid C-string in EVTC Agent data")) }
    }

    pub fn subgroup(&self) -> &str {
        // All agent and skill names use UTF8 according to deltaconnected
        unsafe { str::from_utf8_unchecked(self.name.split(|&c| c == 0).nth(2).expect("Invalid C-string in EVTC Agent data")) }
    }

    pub fn profession_inner(&self) -> u32 {
        self.profession
    }

    pub fn profession(&self) -> Profession {
        match (self.is_elite, self.profession) {
            (0xFFFFFFFF, x)  => if x & 0xffff0000 == 0xffff0000 {
                Profession::Gadget(SpeciesId((x & 0xffff) as u16))
            } else {
                Profession::NonPlayableCharacter(SpeciesId((x & 0xffff) as u16))
            },
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
}

/// Skill id and name.
#[repr(packed)]
#[derive(Copy, Clone)]
pub struct Skill {
    id:   u32,
    name: [u8; 64],
}

impl fmt::Debug for Skill {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Skill({}, {})", {self.id}, self.name())
    }
}

impl Skill {
    /// The skill id.
    #[inline]
    pub fn id(&self) -> u32 {
        self.id
    }

    /// Name of the skill.
    pub fn name(&self) -> &str {
        // All strings are UTF-8
        unsafe { str::from_utf8_unchecked(self.name.split(|&c| c == 0).next().expect("Invalid C-string in EVTC Skill data")) }
    }
}

/// Language constants.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub enum Language {
    English = 0,
    French  = 2,
    German  = 3,
    Spanish = 4,
}

impl Language {
    /// Language is stored as agent id ([u64]).
    #[inline]
    pub fn from_agent_id(id: u64) -> Language {
        match id {
            2 => Language::French,
            3 => Language::German,
            4 => Language::Spanish,
            _ => Language::English,
        }
    }
}

/// If the event targets a friendly (green) or foe (red) agent.
#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IFF {
    /// Targeting a friendly
    Friend  = 0,
    /// Targeting an enemy
    Foe     = 1,
    /// Something is wrong
    Unknown = 2,
}

/// Result of a hit.
#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HitResult {
    /// good physical hit
    Normal      = 0,
    /// physical hit was crit
    Crit        = 1,
    /// physical hit was glance
    Glance      = 2,
    /// physical hit was blocked eg. mesmer shield 4
    Block       = 3,
    /// physical hit was evaded, eg. dodge or mesmer sword 2
    Evade       = 4,
    /// physical hit interrupted something
    Interrupt   = 5,
    /// physical hit was "invlun" or absorbed eg. guardian elite
    Absorb      = 6,
    /// physical hit missed
    Blind       = 7,
    /// physical hit was killing hit
    KillingBlow = 8,
    /// physical hit was downing hit
    DowningBlow = 9,
}

/// Skill activation types types.
#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CombatActivation {
    /// Not used - not this kind of event
    None       = 0,
    /// Without quickness
    Normal     = 1,
    /// With quickness (+50% animation-speed)
    Quickness  = 2,
    /// Cancel with reaching channel time
    CancelFire = 3,
    /// Cancel without reaching channel time
    Cancel     = 4,
    /// Animation completed fully
    Reset      = 5,
}

/// Actor state changes, and also some non-actor state-changes.
#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CombatStateChange {
    /// not used - not this kind of event
    None            = 0,
    /// src_agent entered combat, dst_agent is subgroup
    EnterCombat     = 1,
    /// src_agent left combat
    ExitCombat      = 2,
    /// src_agent is now alive
    ChangeUp        = 3,
    /// src_agent is now dead
    ChangeDead      = 4,
    /// src_agent is now downed
    ChangeDown      = 5,
    /// src_agent is now in game tracking range
    Spawn           = 6,
    /// src_agent is no longer being tracked
    Despawn         = 7,
    /// src_agent has reached a health marker. dst_agent = percent * 10000 (eg. 99.5% will be 9950)
    HealthUpdate    = 8,
    /// log start. value = server unix timestamp **uint32**. buff_dmg = local unix timestamp. src_agent = 0x637261 (arcdps id)
    LogStart        = 9,
    /// log end. value = server unix timestamp **uint32**. buff_dmg = local unix timestamp. src_agent = 0x637261 (arcdps id)
    LogEnd          = 10,
    /// src_agent swapped weapon set. dst_agent = current set id (0/1 water, 4/5 land)
    WeapSwap        = 11,
    /// src_agent has had it's maximum health changed. dst_agent = new max health
    MaxHealthUpdate = 12,
    /// src_agent will be agent of "recording" player
    PointOfView     = 13,
    /// src_agent will be text language
    Language        = 14,
    /// src_agent will be game build
    GwBuild         = 15,
    /// src_agent will be sever shard id
    ShardId         = 16,
    /// src_agent is self, dst_agent is reward id, value is reward type. these are the wiggly boxes that you get
    Reward          = 17,
    /// combat event that will appear once per buff per agent on logging start (zero duration, buff==18)
    BuffInitial     = 18,
    /// src_agent changed, cast float* p = (float*)&dst_agent, access as x/y/z (float\[3\])
    Position        = 19,
    /// src_agent changed, cast float* v = (float*)&dst_agent, access as x/y/z (float\[3\])
    Velocity        = 20,
    /// src_agent changed, cast float* f = (float*)&dst_agent, access as x/y (float\[2\])
    ///
    /// Since 2018-07-18
    Facing          = 21,
}

/// Buff removals.
#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CombatBuffRemove {
    /// Not used - not this kind of event
    None   = 0,
    /// All buff stacks removed
    All    = 1,
    /// Single stack removed. Disabled on server trigger, will happen for each stack on cleanse
    Single = 2,
    /// Autoremoved by outofcombat or allstack (ignore for strip/cleanse calc, use for in/out volume)
    Manual = 3,
}

/// A combat event, version 1, should be used when [Header::combat_data_version] is
/// [CombatDataVersion::V1].
// TODO: Implement all public stuff as a trait
#[repr(packed)]
#[derive(Debug, Clone, Copy)]
pub struct CombatEventV1 {
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
    // 2018-07-10: zero if on-tick dmg, non-zero otherwise (buff only)
    is_offcycle:       u8,
    _pad2:             u8,
}

impl CombatEventV1 {
    fn src_agent(&self)         -> AgentId { AgentId::new(self.src_agent) }
    fn dst_agent(&self)         -> AgentId { AgentId::new(self.dst_agent) }
    fn src_instid(&self)        -> InstanceId { InstanceId::new(self.src_instid) }
    fn dst_instid(&self)        -> InstanceId { InstanceId::new(self.dst_instid) }
    /// Events without a source agent.
    fn is_meta(&self) -> bool {
        match self.is_statechange {
            CombatStateChange::Language |
            CombatStateChange::GwBuild  |
            CombatStateChange::ShardId  |
            CombatStateChange::LogStart |
            CombatStateChange::LogEnd => true,
            _                         => false,
        }
    }
    fn src_master_instid(&self) -> Option<InstanceId> {
        if self.src_master_instid == 0 {
            None
        }
        else {
            Some(InstanceId::new(self.src_master_instid))
        }
    }
}

impl<'a> Event for &'a CombatEventV1 {
    type MetaEvent = MetaEvent<&'a CombatEventV1>;
    type SourceEvent = SourceEvent<&'a CombatEventV1>;
    type DamageEvent = DamageEvent<&'a CombatEventV1>;
    type TargetEvent = TargetEvent<&'a CombatEventV1>;
    type ActivationEvent = ActivationEvent<&'a CombatEventV1>;
    type BuffEvent = BuffEvent<&'a CombatEventV1>;

    #[inline]
    fn time(&self) -> u64 {
        self.time
    }

    #[inline]
    fn into_meta(self) -> Option<Self::MetaEvent> {
        if self.is_meta() {
            Some(MetaEvent(self))
        }
        else {
            None
        }
    }

    #[inline]
    fn into_source(self) -> Option<Self::SourceEvent> {
        if ! self.is_meta() {
            Some(SourceEvent(self))
        }
        else {
            None
        }
    }

    #[inline]
    fn into_damage(self) -> Option<Self::DamageEvent> {
        match (self.is_statechange, self.is_activation, self.is_buffremove, self.buff > 0 && self.buff_dmg == 0) {
            (CombatStateChange::None, CombatActivation::None, CombatBuffRemove::None, false) => Some(DamageEvent(self)),
            _ => None,
        }
    }

    #[inline]
    fn into_activation(self) -> Option<Self::ActivationEvent> {
        if self.is_activation != CombatActivation::None {
            Some(ActivationEvent(self))
        }
        else {
            None
        }
    }

    #[inline]
    fn into_buff(self) -> Option<Self::BuffEvent> {
        if self.is_statechange == CombatStateChange::None &&
           self.is_activation == CombatActivation::None &&
           self.buff > 0 && self.buff_dmg == 0 {
            Some(BuffEvent(self))
        }
        else {
            None
        }
    }

    #[inline]
    fn from_agent(self, a: AgentId) -> Option<Self::SourceEvent> {
        if ! self.is_meta() && self.src_agent() == a {
            Some(SourceEvent(self))
        }
        else {
            None
        }
    }

    #[inline]
    fn from_gadgets(self, master: InstanceId) -> Option<Self::SourceEvent> {
        if ! self.is_meta() && self.src_master_instid() == Some(master) {
            Some(SourceEvent(self))
        }
        else {
            None
        }
    }

    #[inline]
    fn from_agent_or_gadgets(self, a: AgentId, i: InstanceId) -> Option<Self::SourceEvent> {
        if ! self.is_meta() && (self.src_agent() == a || self.src_master_instid() == Some(i)) {
            Some(SourceEvent(self))
        }
        else {
            None
        }
    }

    #[inline]
    fn from_any_of<I: IntoIterator<Item=AgentId>>(self, agents: I) -> Option<Self::SourceEvent> {
        if ! self.is_meta() && agents.into_iter().any(|a| a == self.src_agent()) {
            Some(SourceEvent(self))
        }
        else {
            None
        }
    }

    #[inline]
    fn targeting_agent(self, agent: AgentId) -> Option<Self::TargetEvent> {
        if self.is_statechange == CombatStateChange::None && agent == self.dst_agent() {
            Some(TargetEvent(self))
        }
        else {
            None
        }
    }

    #[inline]
    fn targeting_any_of<I: IntoIterator<Item=AgentId>>(self, agents: I) -> Option<Self::TargetEvent> {
        if self.is_statechange == CombatStateChange::None && agents.into_iter().any(|a| a == self.dst_agent()) {
            Some(TargetEvent(self))
        }
        else {
            None
        }
    }
}

// Event end
//
// MetaEvent start

impl<'a> Event for MetaEvent<&'a CombatEventV1> {
    type MetaEvent = MetaEvent<&'a CombatEventV1>;
    type SourceEvent = SourceEvent<&'a CombatEventV1>;
    type DamageEvent = DamageEvent<&'a CombatEventV1>;
    type TargetEvent = TargetEvent<&'a CombatEventV1>;
    type ActivationEvent = ActivationEvent<&'a CombatEventV1>;
    type BuffEvent = BuffEvent<&'a CombatEventV1>;

    #[inline]
    fn time(&self) -> u64 {
        self.0.time
    }

    #[inline]
    fn into_meta(self) -> Option<Self::MetaEvent> {
        Some(self)
    }

    #[inline]
    fn into_source(self) -> Option<Self::SourceEvent> {
        None
    }

    #[inline]
    fn into_damage(self) -> Option<Self::DamageEvent> {
        None
    }

    #[inline]
    fn into_activation(self) -> Option<Self::ActivationEvent> {
        None
    }

    #[inline]
    fn into_buff(self) -> Option<Self::BuffEvent> {
        None
    }

    #[inline]
    fn from_agent(self, _: AgentId) -> Option<Self::SourceEvent> {
        None
    }

    #[inline]
    fn from_gadgets(self, _: InstanceId) -> Option<Self::SourceEvent> {
        None
    }

    #[inline]
    fn from_any_of<I: IntoIterator<Item=AgentId>>(self, _: I) -> Option<Self::SourceEvent> {
        None
    }

    #[inline]
    fn from_agent_or_gadgets(self, _: AgentId, _: InstanceId) -> Option<Self::SourceEvent> {
        None
    }

    #[inline]
    fn targeting_agent(self, _: AgentId) -> Option<Self::TargetEvent> {
        None
    }

    #[inline]
    fn targeting_any_of<I: IntoIterator<Item=AgentId>>(self, _: I) -> Option<Self::TargetEvent> {
        None
    }
}

impl<'a> Meta for MetaEvent<&'a CombatEventV1> {
    fn into_enum(&self) -> MetaEventData {
        match self.0.is_statechange {
            CombatStateChange::Language => MetaEventData::Language(Language::from_agent_id(self.0.src_agent)),
            CombatStateChange::GwBuild  => MetaEventData::Gw2Build(self.0.src_agent),
            CombatStateChange::ShardId  => MetaEventData::Gw2Build(self.0.src_agent),
            CombatStateChange::LogStart => MetaEventData::LogStart { server: self.0.value as u32, local: self.0.buff_dmg as u32, arcdps_id: self.0.src_agent },
            CombatStateChange::LogEnd   => MetaEventData::LogEnd { server: self.0.value as u32, local: self.0.buff_dmg as u32, arcdps_id: self.0.src_agent },
            _ => unreachable!(),
        }
    }
}

// MetaEvent end
//
// SourceEvent start

impl<'a> Event for SourceEvent<&'a CombatEventV1> {
    type MetaEvent = MetaEvent<&'a CombatEventV1>;
    type SourceEvent = SourceEvent<&'a CombatEventV1>;
    type DamageEvent = DamageEvent<&'a CombatEventV1>;
    type TargetEvent = TargetEvent<&'a CombatEventV1>;
    type ActivationEvent = ActivationEvent<&'a CombatEventV1>;
    type BuffEvent = BuffEvent<&'a CombatEventV1>;

    #[inline]
    fn time(&self) -> u64 {
        self.0.time
    }

    #[inline]
    fn into_meta(self) -> Option<Self::MetaEvent> {
        None
    }

    #[inline]
    fn into_source(self) -> Option<Self::SourceEvent> {
        Some(self)
    }

    #[inline]
    fn into_damage(self) -> Option<Self::DamageEvent> {
        match (self.0.is_statechange, self.0.is_activation, self.0.is_buffremove, self.0.buff > 0 && self.0.buff_dmg == 0) {
            (CombatStateChange::None, CombatActivation::None, CombatBuffRemove::None, false) => Some(DamageEvent(self.0)),
            _ => None,
        }
    }

    #[inline]
    fn into_activation(self) -> Option<Self::ActivationEvent> {
        if self.0.is_activation == CombatActivation::None {
            None
        }
        else {
            Some(ActivationEvent(self.0))
        }
    }

    #[inline]
    fn into_buff(self) -> Option<Self::BuffEvent> {
        if self.0.is_statechange == CombatStateChange::None &&
           self.0.is_activation == CombatActivation::None &&
           self.0.buff > 0 && self.0.buff_dmg == 0 {
            Some(BuffEvent(self.0))
        }
        else {
            None
        }
    }

    #[inline]
    fn from_agent(self, a: AgentId) -> Option<Self::SourceEvent> {
        if self.0.src_agent() == a {
            Some(self)
        }
        else {
            None
        }
    }

    #[inline]
    fn from_gadgets(self, master: InstanceId) -> Option<Self::SourceEvent> {
        if self.0.src_master_instid() == Some(master) {
            Some(self)
        }
        else {
            None
        }
    }

    #[inline]
    fn from_agent_or_gadgets(self, a: AgentId, i: InstanceId) -> Option<Self::SourceEvent> {
        if self.0.src_agent() == a || self.0.src_master_instid() == Some(i) {
            Some(self)
        }
        else {
            None
        }
    }

    #[inline]
    fn from_any_of<I: IntoIterator<Item=AgentId>>(self, agents: I) -> Option<Self::SourceEvent> {
        if agents.into_iter().any(|a| a == self.0.src_agent()) {
            Some(SourceEvent(self.0))
        }
        else {
            None
        }
    }

    #[inline]
    fn targeting_agent(self, agent: AgentId) -> Option<Self::TargetEvent> {
        // TODO: Fix potential overlap with some state-changes
        if agent == self.0.dst_agent() {
            Some(TargetEvent(self.0))
        }
        else {
            None
        }
    }

    #[inline]
    fn targeting_any_of<I: IntoIterator<Item=AgentId>>(self, agents: I) -> Option<Self::TargetEvent> {
        // TODO: Fix potential overlap with some state-changes
        if agents.into_iter().any(|a| a == self.0.dst_agent()) {
            Some(TargetEvent(self.0))
        }
        else {
            None
        }
    }
}

impl<'a> Source for SourceEvent<&'a CombatEventV1> {
    #[inline]
    fn agent(&self) -> AgentId {
        self.0.src_agent()
    }

    #[inline]
    fn instance(&self) -> InstanceId {
        self.0.src_instid()
    }

    #[inline]
    fn master_instance(&self) -> Option<InstanceId> {
        self.0.src_master_instid()
    }

    #[inline]
    fn state_change(&self) -> Option<StateChange> {
        match self.0.is_statechange {
            CombatStateChange::EnterCombat     => Some(StateChange::EnterCombat(self.0.dst_agent)),
            CombatStateChange::ExitCombat      => Some(StateChange::ExitCombat),
            CombatStateChange::ChangeUp        => Some(StateChange::ChangeUp),
            CombatStateChange::ChangeDead      => Some(StateChange::ChangeDead),
            CombatStateChange::ChangeDown      => Some(StateChange::ChangeDown),
            CombatStateChange::Spawn           => Some(StateChange::Spawn),
            CombatStateChange::Despawn         => Some(StateChange::Despawn),
            CombatStateChange::HealthUpdate    => Some(StateChange::HealthUpdate(self.0.dst_agent)),
            CombatStateChange::WeapSwap        => Some(StateChange::WeaponSwap),
            CombatStateChange::MaxHealthUpdate => Some(StateChange::MaxHealthUpdate(self.0.dst_agent)),
            CombatStateChange::PointOfView     => Some(StateChange::PointOfView),
            CombatStateChange::Reward          => Some(StateChange::Reward(self.0.dst_agent, self.0.value as u32)),
            CombatStateChange::BuffInitial     => {
                // TODO: What is this?
                Some(StateChange::BuffInitial)
            },
            CombatStateChange::Position        => {
                let pos: &[f32; 3] = unsafe { mem::transmute(&self.0.dst_agent) };

                Some(StateChange::Position { x: pos[0], y: pos[1], z: pos[2] })
            },
            CombatStateChange::Velocity        => {
                let pos: &[f32; 3] = unsafe { mem::transmute(&self.0.dst_agent) };

                Some(StateChange::Position { x: pos[0], y: pos[1], z: pos[2] })
            },
            CombatStateChange::Facing        => {
                let pos: &[f32; 2] = unsafe { mem::transmute(&self.0.dst_agent) };

                Some(StateChange::Facing { x: pos[0], y: pos[1] })
            },
            _ => None
        }
    }
}

// SourceEvent end
//
// TargetEvent start

impl<'a> Event for TargetEvent<&'a CombatEventV1> {
    type MetaEvent = MetaEvent<&'a CombatEventV1>;
    type SourceEvent = TargetEvent<&'a CombatEventV1>;
    type DamageEvent = DamageEvent<&'a CombatEventV1>;
    type TargetEvent = TargetEvent<&'a CombatEventV1>;
    type ActivationEvent = ActivationEvent<&'a CombatEventV1>;
    type BuffEvent = BuffEvent<&'a CombatEventV1>;

    #[inline]
    fn time(&self) -> u64 {
        self.0.time
    }

    #[inline]
    fn into_meta(self) -> Option<Self::MetaEvent> {
        None
    }

    #[inline]
    fn into_source(self) -> Option<Self::SourceEvent> {
        Some(self)
    }

    #[inline]
    fn into_damage(self) -> Option<Self::DamageEvent> {
        // Target events should not have any statechanges or activations
        debug_assert!(self.0.is_statechange == CombatStateChange::None);
        debug_assert!(self.0.is_activation == CombatActivation::None);

        if self.0.is_buffremove == CombatBuffRemove::None &&
            (self.0.buff == 0 || self.0.buff_dmg != 0) {
            Some(DamageEvent(self.0))
        }
        else {
            None
        }
    }

    #[inline]
    fn into_activation(self)   -> Option<Self::ActivationEvent> {
        // Target-events can't be activations
        None
    }

    #[inline]
    fn into_buff(self) -> Option<Self::BuffEvent> {
        // Target events should not have any statechanges or activations
        debug_assert!(self.0.is_statechange == CombatStateChange::None);
        debug_assert!(self.0.is_activation == CombatActivation::None);

        if self.0.buff > 0 && self.0.buff_dmg == 0 {
            Some(BuffEvent(self.0))
        }
        else {
            None
        }
    }

    #[inline]
    fn from_agent(self, a: AgentId) -> Option<Self::SourceEvent> {
        if self.0.src_agent() == a {
            Some(self)
        }
        else {
            None
        }
    }

    #[inline]
    fn from_gadgets(self, master: InstanceId) -> Option<Self::SourceEvent> {
        if self.0.src_master_instid() == Some(master) {
            Some(self)
        }
        else {
            None
        }
    }

    #[inline]
    fn from_agent_or_gadgets(self, a: AgentId, i: InstanceId) -> Option<Self::SourceEvent> {
        if self.0.src_agent() == a || self.0.src_master_instid() == Some(i) {
            Some(self)
        }
        else {
            None
        }
    }

    #[inline]
    fn from_any_of<I: IntoIterator<Item=AgentId>>(self, agents: I) -> Option<Self::SourceEvent> {
        if agents.into_iter().any(|a| a == self.0.src_agent()) {
            Some(self)
        }
        else {
            None
        }
    }

    #[inline]
    fn targeting_agent(self, agent: AgentId) -> Option<Self::TargetEvent> {
        if agent == self.0.dst_agent() {
            Some(TargetEvent(self.0))
        }
        else {
            None
        }
    }

    #[inline]
    fn targeting_any_of<I: IntoIterator<Item=AgentId>>(self, agents: I) -> Option<Self::TargetEvent> {
        if agents.into_iter().any(|a| a == self.0.dst_agent()) {
            Some(self)
        }
        else {
            None
        }
    }
}


impl<'a> Source for TargetEvent<&'a CombatEventV1> {
    #[inline]
    fn agent(&self) -> AgentId {
        self.0.src_agent()
    }

    #[inline]
    fn instance(&self) -> InstanceId {
        self.0.src_instid()
    }

    #[inline]
    fn master_instance(&self) -> Option<InstanceId> {
        self.0.src_master_instid()
    }

    #[inline]
    fn state_change(&self) -> Option<StateChange> {
        None
    }
}

impl<'a> Target for TargetEvent<&'a CombatEventV1> {
    #[inline]
    fn target_agent(&self) -> AgentId {
        AgentId::new(self.0.dst_agent)
    }

    #[inline]
    fn target_instance(&self) -> InstanceId {
        InstanceId::new(self.0.dst_instid)
    }
}

// TargetEvent end
//
// ActivationEvent start

impl<'a> Event for ActivationEvent<&'a CombatEventV1> {
    type MetaEvent = MetaEvent<&'a CombatEventV1>;
    type SourceEvent = ActivationEvent<&'a CombatEventV1>;
    type DamageEvent = DamageEvent<&'a CombatEventV1>;
    type TargetEvent = TargetEvent<&'a CombatEventV1>;
    type ActivationEvent = ActivationEvent<&'a CombatEventV1>;
    type BuffEvent = BuffEvent<&'a CombatEventV1>;

    #[inline]
    fn time(&self) -> u64 {
        self.0.time
    }

    #[inline]
    fn into_meta(self) -> Option<Self::MetaEvent> {
        None
    }

    #[inline]
    fn into_source(self) -> Option<Self::SourceEvent> {
        Some(self)
    }

    #[inline]
    fn into_damage(self) -> Option<Self::DamageEvent> {
        // Skill activations can't do damage
        None
    }

    #[inline]
    fn into_activation(self)   -> Option<Self::ActivationEvent> {
        Some(self)
    }

    #[inline]
    fn into_buff(self) -> Option<Self::BuffEvent> {
        None
    }

    #[inline]
    fn from_agent(self, a: AgentId) -> Option<Self::SourceEvent> {
        if self.0.src_agent() == a {
            Some(self)
        }
        else {
            None
        }
    }

    #[inline]
    fn from_gadgets(self, master: InstanceId) -> Option<Self::SourceEvent> {
        if self.0.src_master_instid() == Some(master) {
            Some(self)
        }
        else {
            None
        }
    }

    #[inline]
    fn from_agent_or_gadgets(self, a: AgentId, i: InstanceId) -> Option<Self::SourceEvent> {
        if self.0.src_agent() == a || self.0.src_master_instid() == Some(i) {
            Some(self)
        }
        else {
            None
        }
    }

    #[inline]
    fn from_any_of<I: IntoIterator<Item=AgentId>>(self, agents: I) -> Option<Self::SourceEvent> {
        if agents.into_iter().any(|a| a == self.0.src_agent()) {
            Some(self)
        }
        else {
            None
        }
    }

    #[inline]
    fn targeting_agent(self, _: AgentId) -> Option<Self::TargetEvent> {
        // Activation events do not have a target
        None
    }

    #[inline]
    fn targeting_any_of<I: IntoIterator<Item=AgentId>>(self, _: I) -> Option<Self::TargetEvent> {
        // Activation events do not have a target
        None
    }
}


impl<'a> Source for ActivationEvent<&'a CombatEventV1> {
    #[inline]
    fn agent(&self) -> AgentId {
        self.0.src_agent()
    }

    #[inline]
    fn instance(&self) -> InstanceId {
        self.0.src_instid()
    }

    #[inline]
    fn master_instance(&self) -> Option<InstanceId> {
        self.0.src_master_instid()
    }

    #[inline]
    fn state_change(&self) -> Option<StateChange> {
        None
    }
}

impl<'a> Activation for ActivationEvent<&'a CombatEventV1> {
    #[inline]
    fn skill(&self) -> u16 {
        self.0.skill_id
    }

    #[inline]
    fn cast(&self) -> CastType {
        match self.0.is_activation {
            CombatActivation::Normal     => CastType::Normal(self.0.value as u32),
            CombatActivation::Quickness  => CastType::Quickness(self.0.value as u32),
            CombatActivation::CancelFire => CastType::CancelFire(self.0.value as u32),
            CombatActivation::Cancel     => CastType::Cancel(self.0.value as u32),
            CombatActivation::Reset      => CastType::Reset,
            // Should have been filtered out
            CombatActivation::None       => unreachable!(),
        }
    }
}

// ActivationEvent end
//
// DamageEvent start

impl<'a> Event for DamageEvent<&'a CombatEventV1> {
    type MetaEvent = MetaEvent<&'a CombatEventV1>;
    type SourceEvent = DamageEvent<&'a CombatEventV1>;
    type DamageEvent = DamageEvent<&'a CombatEventV1>;
    type TargetEvent = DamageEvent<&'a CombatEventV1>;
    type ActivationEvent = ActivationEvent<&'a CombatEventV1>;
    type BuffEvent = BuffEvent<&'a CombatEventV1>;

    #[inline]
    fn time(&self) -> u64 {
        self.0.time
    }

    #[inline]
    fn into_meta(self) -> Option<Self::MetaEvent> {
        None
    }

    #[inline]
    fn into_source(self) -> Option<Self::SourceEvent> {
        Some(self)
    }

    #[inline]
    fn into_damage(self) -> Option<Self::DamageEvent> {
        Some(self)
    }

    #[inline]
    fn into_activation(self)   -> Option<Self::ActivationEvent> {
        // Target-events can't be activations
        None
    }

    #[inline]
    fn into_buff(self) -> Option<Self::BuffEvent> {
        None
    }

    #[inline]
    fn from_agent(self, a: AgentId) -> Option<Self::SourceEvent> {
        if self.0.src_agent() == a {
            Some(self)
        }
        else {
            None
        }
    }

    #[inline]
    fn from_gadgets(self, master: InstanceId) -> Option<Self::SourceEvent> {
        if self.0.src_master_instid() == Some(master) {
            Some(self)
        }
        else {
            None
        }
    }

    #[inline]
    fn from_agent_or_gadgets(self, a: AgentId, i: InstanceId) -> Option<Self::SourceEvent> {
        if self.0.src_agent() == a || self.0.src_master_instid() == Some(i) {
            Some(self)
        }
        else {
            None
        }
    }

    #[inline]
    fn from_any_of<I: IntoIterator<Item=AgentId>>(self, agents: I) -> Option<Self::SourceEvent> {
        if agents.into_iter().any(|a| a == self.0.src_agent()) {
            Some(self)
        }
        else {
            None
        }
    }

    #[inline]
    fn targeting_agent(self, agent: AgentId) -> Option<Self::TargetEvent> {
        if agent == self.0.dst_agent() {
            Some(self)
        }
        else {
            None
        }
    }

    #[inline]
    fn targeting_any_of<I: IntoIterator<Item=AgentId>>(self, agents: I) -> Option<Self::TargetEvent> {
        if agents.into_iter().any(|a| a == self.0.dst_agent()) {
            Some(self)
        }
        else {
            None
        }
    }
}

impl<'a> Source for DamageEvent<&'a CombatEventV1> {
    #[inline]
    fn agent(&self) -> AgentId {
        self.0.src_agent()
    }

    #[inline]
    fn instance(&self) -> InstanceId {
        self.0.src_instid()
    }

    #[inline]
    fn master_instance(&self) -> Option<InstanceId> {
        self.0.src_master_instid()
    }

    #[inline]
    fn state_change(&self) -> Option<StateChange> {
        None
    }
}

impl<'a> Target for DamageEvent<&'a CombatEventV1> {
    #[inline]
    fn target_agent(&self) -> AgentId {
        self.0.dst_agent()
    }

    #[inline]
    fn target_instance(&self) -> InstanceId {
        self.0.dst_instid()
    }
}

impl<'a> Damage for DamageEvent<&'a CombatEventV1> {
    #[inline]
    fn skill(&self) -> u16 {
        self.0.skill_id
    }

    #[inline]
    fn damage(&self) -> i64 {
        if self.0.buff > 0 {
            // Resisted damage seems to be negative
            cmp::max(0, self.0.buff_dmg as i64)
        }
        else {
            self.0.value as i64
        }
    }

    #[inline]
    fn flanking(&self) -> bool {
        self.0.is_flanking > 0
    }

    #[inline]
    fn moving(&self) -> bool {
        self.0.is_src_moving > 0
    }

    #[inline]
    fn hit_type(&self) -> HitType {
        match (self.0.buff > 0, self.0.result) {
            (true, _)                      => HitType::Condi,
            (_,    HitResult::Normal)      => HitType::Normal,
            (_,    HitResult::Crit)        => HitType::Crit,
            (_,    HitResult::Glance)      => HitType::Glance,
            (_,    HitResult::Block)       => HitType::Block,
            (_,    HitResult::Evade)       => HitType::Evade,
            (_,    HitResult::Interrupt)   => HitType::Interrupt,
            (_,    HitResult::Absorb)      => HitType::Absorb,
            (_,    HitResult::Blind)       => HitType::Blind,
            (_,    HitResult::KillingBlow) => HitType::KillingBlow,
            (_,    HitResult::DowningBlow) => HitType::DowningBlow,
        }
    }

    #[inline]
    fn over90(&self) -> bool {
        self.0.is_src_ninety > 0
    }
}

// DamageEvent end
//
// BuffEvent start

impl<'a> Event for BuffEvent<&'a CombatEventV1> {
    type MetaEvent = MetaEvent<&'a CombatEventV1>;
    type SourceEvent = BuffEvent<&'a CombatEventV1>;
    type DamageEvent = DamageEvent<&'a CombatEventV1>;
    type TargetEvent = BuffEvent<&'a CombatEventV1>;
    type ActivationEvent = ActivationEvent<&'a CombatEventV1>;
    type BuffEvent = BuffEvent<&'a CombatEventV1>;

    #[inline]
    fn time(&self) -> u64 {
        self.0.time
    }

    #[inline]
    fn into_meta(self) -> Option<Self::MetaEvent> {
        None
    }

    #[inline]
    fn into_source(self) -> Option<Self::SourceEvent> {
        Some(self)
    }

    #[inline]
    fn into_damage(self) -> Option<Self::DamageEvent> {
        None
    }

    #[inline]
    fn into_activation(self)   -> Option<Self::ActivationEvent> {
        // Target-events can't be activations
        None
    }

    #[inline]
    fn into_buff(self) -> Option<Self::BuffEvent> {
        Some(self)
    }

    #[inline]
    fn from_agent(self, a: AgentId) -> Option<Self::SourceEvent> {
        if self.0.src_agent() == a {
            Some(self)
        }
        else {
            None
        }
    }

    #[inline]
    fn from_gadgets(self, master: InstanceId) -> Option<Self::SourceEvent> {
        if self.0.src_master_instid() == Some(master) {
            Some(self)
        }
        else {
            None
        }
    }

    #[inline]
    fn from_agent_or_gadgets(self, a: AgentId, i: InstanceId) -> Option<Self::SourceEvent> {
        if self.0.src_agent() == a || self.0.src_master_instid() == Some(i) {
            Some(self)
        }
        else {
            None
        }
    }

    #[inline]
    fn from_any_of<I: IntoIterator<Item=AgentId>>(self, agents: I) -> Option<Self::SourceEvent> {
        if agents.into_iter().any(|a| a == self.0.src_agent()) {
            Some(self)
        }
        else {
            None
        }
    }

    #[inline]
    fn targeting_agent(self, agent: AgentId) -> Option<Self::TargetEvent> {
        if agent == self.0.dst_agent() {
            Some(self)
        }
        else {
            None
        }
    }

    #[inline]
    fn targeting_any_of<I: IntoIterator<Item=AgentId>>(self, agents: I) -> Option<Self::TargetEvent> {
        if agents.into_iter().any(|a| a == self.0.dst_agent()) {
            Some(self)
        }
        else {
            None
        }
    }
}

impl<'a> Source for BuffEvent<&'a CombatEventV1> {
    #[inline]
    fn agent(&self) -> AgentId {
        self.0.src_agent()
    }

    #[inline]
    fn instance(&self) -> InstanceId {
        self.0.src_instid()
    }

    #[inline]
    fn master_instance(&self) -> Option<InstanceId> {
        self.0.src_master_instid()
    }

    #[inline]
    fn state_change(&self) -> Option<StateChange> {
        None
    }
}

impl<'a> Target for BuffEvent<&'a CombatEventV1> {
    #[inline]
    fn target_agent(&self) -> AgentId {
        self.0.dst_agent()
    }

    #[inline]
    fn target_instance(&self) -> InstanceId {
        self.0.dst_instid()
    }
}

impl<'a> Buff for BuffEvent<&'a CombatEventV1> {
    #[inline]
    fn skill(&self) -> u16 {
        self.0.skill_id
    }

    #[inline]
    fn duration(&self) -> u32 {
        self.0.value as u32
    }

    #[inline]
    fn overstack(&self) -> u32 {
        self.0.overstack as u32
    }

    #[inline]
    fn removal(&self) -> CombatBuffRemove {
        self.0.is_buffremove
    }
}

// BuffEvent end

/// An owning buffer with copies of all the data.
#[derive(Debug)]
pub struct Evtc {
    pub header: Header,
    pub agents: Vec<Agent>,
    pub skills: Vec<Skill>,
    pub events: Vec<CombatEventV1>,
}

/// A buffer borrowing from a slice of raw EVTC-data.
#[derive(Debug)]
#[derive(Copy, Clone)]
pub struct EvtcBuf<'a> {
    pub header: &'a Header,
    pub agents: &'a [Agent],
    pub skills: &'a [Skill],
    pub events: &'a [CombatEventV1],
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

/// Transmutes a buffer into an [EvtcBuf].
// TODO: Handle errors instead of panic
pub fn transmute(buffer: &[u8]) -> EvtcBuf {
    let (header, buffer)     = transmute_single::<Header>(buffer).expect("EVTC-data is missing header");
    let (agents, buffer)     = transmute_slice(buffer, header.agents as usize).expect("EVTC-data too short, failed to read agents");
    let (num_skills, buffer) = transmute_single::<u32>(buffer).expect("EVTC-data too short, missing skill count");
    let (skills, buffer)     = transmute_slice(buffer, *num_skills as usize).expect("EVTC-data too short, failed to extract skills");
    let (events, _buffer)    = transmute_slice(buffer, buffer.len() / mem::size_of::<CombatEventV1>()).expect("EVTC-data too short, failed to extract events");

    EvtcBuf {
        header,
        agents,
        skills,
        events,
    }
}