use types::AgentId;
use types::InstanceId;

pub use self::raw::Language;

pub mod raw;

pub struct EventMarker;

pub trait Event: Clone {
    type MetaEvent:   Meta;
    type SourceEvent: Source;
    type TargetEvent: Target;
    type ActivationEvent: Activation;
    type DamageEvent: Damage;

    fn time(&self) -> u64;
    fn into_source(self) -> Option<Self::SourceEvent>;
    fn into_meta(self) -> Option<Self::MetaEvent>;
    fn into_damage(self) -> Option<Self::DamageEvent>;
    fn from_agent(self, AgentId) -> Option<Self::SourceEvent>;
    fn from_gadgets(self, InstanceId) -> Option<Self::SourceEvent>;
    fn from_agent_or_gadgets(self, AgentId, InstanceId) -> Option<Self::SourceEvent>;
    fn from_any_of<I: IntoIterator<Item=AgentId>>(self, I) -> Option<Self::SourceEvent>;
    fn targeting_any_of<I: IntoIterator<Item=AgentId>>(self, I) -> Option<Self::TargetEvent>;
}

pub trait Meta: Event {
    fn into_enum(&self) -> MetaEventData;
}

pub trait Source: Event {
    fn agent(&self) -> AgentId;
    fn instance(&self) -> InstanceId;
    fn master_instance(&self) -> Option<InstanceId>;
    fn state_change(&self) -> Option<StateChange>;
    fn into_activation(self) -> Option<Self::ActivationEvent>;
    // TODO: Buff event
}


pub trait Activation: Source {
    fn skill(&self) -> u16;
    fn cast(&self)  -> CastType;
}

pub trait Target: Source {
    fn target_agent(&self)    -> AgentId;
    fn target_instance(&self) -> InstanceId;
}

pub trait Buff: Target {

}

pub trait Damage: Target<SourceEvent=Self, TargetEvent=Self, DamageEvent=Self>
  where Self: Sized {
    fn skill(&self)    -> u16;
    fn damage(&self)   -> i64;
    fn flanking(&self) -> bool;
    fn moving(&self)   -> bool;
    fn hit_type(&self) -> HitType;
    fn over90(&self)   -> bool;
}

#[derive(Debug, Clone)]
pub struct MetaEvent<T>(T);

#[derive(Debug, Clone)]
pub struct SourceEvent<T>(T);

#[derive(Debug, Clone)]
pub struct ActivationEvent<T>(T);

#[derive(Debug, Clone)]
pub struct DamageEvent<T>(T);

#[derive(Debug, Clone)]
pub struct TargetEvent<T>(T);

#[derive(Debug, Clone)]
pub struct BuffEvent<T>(T);

#[derive(Debug, Copy, Clone)]
pub enum MetaEventData {
    /// Server unix timestamp, local unix timestamp, arcdpsId
    LogStart { server: u32, local: u32, arcdps_id: u64 },
    LogEnd   { server: u32, local: u32, arcdps_id: u64 },
    Language(Language),
    Gw2Build(u64),
    ShardId(u64),
}

#[derive(Debug, Copy, Clone)]
pub enum HitType {
    Condi,
    Normal,
    Crit,
    Glance,
    Block,
    Evade,
    Interrupt,
    Absorb,
    Blind,
    KillingBlow,
}

impl HitType {
    pub fn is_zero(self) -> bool {
        match self {
            HitType::Block | HitType::Evade | HitType::Interrupt | HitType::Absorb | HitType::Blind => true,
            _ => false,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum StateChange {
    EnterCombat(u64),
    ExitCombat,
    ChangeUp,
    ChangeDead,
    ChangeDown,
    Spawn,
    Despawn,
    /// Agent has a health-update, value is % * 10000 (eg. 99.5% will be 9950)
    HealthUpdate(u64),
    WeaponSwap,
    MaxHealthUpdate(u64),
    PointOfView,
    /// Wiggly boxes, reward id and reward type
    Reward(u64, u32),
    Position { x: f32, y: f32, z: f32 },
    Velocity { x: f32, y: f32, z: f32 },
    /// Happens once per agent on start
    // TODO: What is this? Should have more data
    BuffInitial,
}

#[derive(Debug, Copy, Clone)]
pub enum CastType {
    // Normal cast, expected duration
    Normal(u32),
    // Fast cast (+50%), expected duration
    Quickness(u32),
    // Canceled but started channel, actual duration
    CancelFire(u32),
    // Canceled before channel, actual duration
    Cancel(u32),
    Reset,
}

impl CastType {
    #[inline]
    pub fn duration(&self) -> u32 {
        match *self {
            CastType::Normal(d)     => d,
            CastType::Quickness(d)  => d,
            CastType::CancelFire(d) => d,
            CastType::Cancel(d)     => d,
            CastType::Reset         => 0,
        }
    }
}