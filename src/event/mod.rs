//!
//! ## Buff events
//!
//! In buff events the source agent (returned from [Source::agent], [Source::instance] and filtered
//! by [Event::from_agent], [Event::targeting_agent] and similar filters) is the agent being
//! dispelled if it is a buff removal. Always cast into a [Buff] first using [Event::into_buff],
//! then check using [Buff::is_remove] to decide which filter/agent to use.
use AgentId;
use InstanceId;

pub use self::raw::Language;

pub use self::raw::CombatBuffRemove as BuffRemoval;

pub mod raw;

/// Basic event type, contains methods for accessing data common to all events and to refine the
/// event into a more specific type.
pub trait Event: Clone {
    /// Meta event type, usually a [MetaEvent] wrapper around the type implementing [Event].
    type MetaEvent: Meta;
    /// Source event type, usually a [SourceEvent] wrapper around the type implementing [Event].
    type SourceEvent: Source;
    /// Target event type, usually a [TargetEvent] wrapper around the type implementing [Event].
    type TargetEvent: Target;
    /// Activation event type, usually a [ActivationEvent] wrapper around the type implementing [Event].
    type ActivationEvent: Activation;
    /// Damage event type, usually a [DamageEvent] wrapper around the type implementing [Event].
    type DamageEvent: Damage;
    /// Buff event type, usually a [BuffEvent] wrapper around the type implementing [Event].
    type BuffEvent: Buff;

    /// Timestamp of the event in milliseconds, relative time of PoV.
    fn time(&self) -> u64;
    fn into_source(self) -> Option<Self::SourceEvent>;
    fn into_meta(self) -> Option<Self::MetaEvent>;
    fn into_damage(self) -> Option<Self::DamageEvent>;
    fn into_activation(self) -> Option<Self::ActivationEvent>;
    fn into_buff(self) -> Option<Self::BuffEvent>;
    fn from_agent(self, AgentId) -> Option<Self::SourceEvent>;
    // TODO: Are InstanceIds reused?
    fn from_gadgets(self, InstanceId) -> Option<Self::SourceEvent>;
    // TODO: Are InstanceIds reused?
    fn from_agent_or_gadgets(self, AgentId, InstanceId) -> Option<Self::SourceEvent>;
    fn from_any_of<I: IntoIterator<Item=AgentId>>(self, I) -> Option<Self::SourceEvent>;
    fn targeting_agent(self, AgentId) -> Option<Self::TargetEvent>;
    fn targeting_any_of<I: IntoIterator<Item=AgentId>>(self, I) -> Option<Self::TargetEvent>;
}

/// Trait for events which are not tied to any source agent.
pub trait Meta: Event<MetaEvent=Self> {
    fn into_enum(&self) -> MetaEventData;
}

/// Trait for events which are tied to a source agent.
pub trait Source: Event<SourceEvent=Self> {
    /// The source of the event.
    fn agent(&self) -> AgentId;
    fn instance(&self) -> InstanceId;
    fn master_instance(&self) -> Option<InstanceId>;
    fn state_change(&self) -> Option<StateChange>;
}

/// Trait for skill-casts, where agents activate skills.
pub trait Activation: Source<SourceEvent=Self, ActivationEvent=Self> {
    fn skill(&self) -> u16;
    fn cast(&self)  -> CastType;
}

/// Trait for events which affect a target.
pub trait Target: Source<SourceEvent=Self, TargetEvent=Self> {
    fn target_agent(&self)    -> AgentId;
    fn target_instance(&self) -> InstanceId;
}

/// Trait for events which apply a buff/boon/debuff/condition to a target.
pub trait Buff: Target<SourceEvent=Self, TargetEvent=Self, BuffEvent=Self> {
    // TODO: Move skill to Target?
    fn skill(&self) -> u16;
    fn removal(&self) -> BuffRemoval;
    fn duration(&self) -> u32;
    fn overstack(&self) -> u32;

    fn is_remove(&self) -> bool {
        self.removal() != BuffRemoval::None
    }
}

/// Trait for events which damage a target.
pub trait Damage: Target<SourceEvent=Self, TargetEvent=Self, DamageEvent=Self>
  where Self: Sized {
    fn skill(&self)    -> u16;
    fn damage(&self)   -> i64;
    fn flanking(&self) -> bool;
    fn moving(&self)   -> bool;
    fn hit_type(&self) -> HitType;
    fn over90(&self)   -> bool;
}

/// Wrapper around an event indicating that the event is a meta-event.
#[derive(Debug, Clone)]
pub struct MetaEvent<T: Event>(T);

/// Wrapper around an event to indicate that it has a source.
#[derive(Debug, Clone)]
pub struct SourceEvent<T: Event>(T);

/// Wrapper around an event to indicate it is an activation event.
#[derive(Debug, Clone)]
pub struct ActivationEvent<T: Event>(T);

/// Wrapper around an event to indicate that it is a damage event.
#[derive(Debug, Clone)]
pub struct DamageEvent<T: Event>(T);

/// Wrapper around an event to indicate that it has a target.
#[derive(Debug, Clone)]
pub struct TargetEvent<T: Event>(T);

/// Wrapper around an event to indicate that it is a buff application/removal event.
#[derive(Debug, Clone)]
pub struct BuffEvent<T: Event>(T);

/// Data not tied to any agent.
#[derive(Debug, Copy, Clone)]
pub enum MetaEventData {
    /// When the log starts, server unix timestamp, local unix timestamp, arcdpsId
    LogStart { server: u32, local: u32, arcdps_id: u64 },
    /// When the log ends, server unix timestamp, local unix timestamp, arcdpsId
    LogEnd   { server: u32, local: u32, arcdps_id: u64 },
    /// The language used of the client
    Language(Language),
    /// The Guild Wars 2 build id
    Gw2Build(u64),
    /// The Guild Wars 2 server shard id
    ShardId(u64),
}

/// The type of damaging hit.
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
    DowningBlow,
}

impl HitType {
    /// True if the hit type should result in zero damage.
    pub fn is_zero(self) -> bool {
        match self {
            HitType::Block | HitType::Evade | HitType::Interrupt | HitType::Absorb | HitType::Blind => true,
            _ => false,
        }
    }
}

/// State updates for agents.
#[derive(Debug, Copy, Clone)]
pub enum StateChange {
    /// Agent entered combat.
    EnterCombat(u64),
    /// Agent exited combat.
    ExitCombat,
    /// Agent got rallied.
    ChangeUp,
    /// Agent died.
    ChangeDead,
    /// Agent got downed.
    ChangeDown,
    /// Agent spawned.
    Spawn,
    /// Agent despawned.
    Despawn,
    /// Agent has a health-update, value is % * 10000 (eg. 99.5% will be 9950)
    HealthUpdate(u64),
    /// Agent swapped weapons.
    WeaponSwap,
    /// Agent got its max health updated.
    MaxHealthUpdate(u64),
    /// Agent is the player recording the log.
    PointOfView,
    /// Wiggly boxes, reward id and reward type
    Reward(u64, u32),
    /// Agent position has been updated.
    Position { x: f32, y: f32, z: f32 },
    /// Agent velocity has been updated.
    Velocity { x: f32, y: f32, z: f32 },
    /// Agent facing has been updated.
    Facing   { x: f32, y: f32 },
    /// Happens once per agent on start
    // TODO: What is this? Should have more data
    BuffInitial,
}

/// Type of skill animation activation.
#[derive(Debug, Copy, Clone)]
pub enum CastType {
    /// Normal cast, expected duration
    Normal(u32),
    /// Fast cast (+50%), expected duration
    Quickness(u32),
    /// Canceled but started channel, actual duration
    CancelFire(u32),
    /// Canceled before channel, actual duration
    Cancel(u32),
    /// Animation completed fully
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
