
use std::mem;
use std::fmt;
use std::u64;

macro_rules! const_assert {
    ($($condition:expr),+ $(,)*) => {
        let _ = [(); 0 - !($($condition)&&+) as usize];
    };
    ($label:ident; $($rest:tt)+) => {
        #[allow(non_snake_case, dead_code)]
        fn $label() {
            const_assert!($($rest)+);
        }
    };
}


#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum EventType {
    StateChange,
    Activation,
    BuffRemove,
    BuffApplication,
    PhysicalHit,
}

#[derive(Clone, Copy, Eq, PartialEq, Debug, Hash)]
pub struct AgentId(u64);

const_assert!(AgentIdSize; mem::size_of::<AgentId>() == 8);

impl fmt::Display for AgentId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "#{}", {self.0})
    }
}

impl Default for AgentId {
    fn default() -> Self {
        AgentId::empty()
    }
}

impl AgentId {
    #[inline(always)]
    pub fn empty() -> Self {
        AgentId(0)
    }

    pub fn new(id: u64) -> Self {
        AgentId(id)
    }
}

#[derive(Clone, Copy, Eq, PartialEq, Debug, Hash, Serialize)]
pub struct InstanceId(u16);

const_assert!(InstanceIdSize; mem::size_of::<InstanceId>() == 2);

impl fmt::Display for InstanceId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{{{}}}", {self.0})
    }
}

impl Default for InstanceId {
    fn default() -> Self {
        InstanceId::empty()
    }
}

impl InstanceId {
    #[inline(always)]
    pub fn empty() -> Self {
        InstanceId(0)
    }

    #[inline(always)]
    pub fn new(id: u16) -> Self {
        InstanceId(id)
    }
}

#[derive(Clone, Copy, Eq, PartialEq, Debug, Hash, Serialize)]
pub struct SpeciesId(u16);

const_assert!(SpeciesIdSize; mem::size_of::<SpeciesId>() == 2);

impl fmt::Display for SpeciesId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "&{}", {self.0})
    }
}

impl Default for SpeciesId {
    fn default() -> Self {
        SpeciesId::empty()
    }
}

impl SpeciesId {
    #[inline(always)]
    pub fn empty() -> Self {
        SpeciesId(0)
    }

    #[inline(always)]
    pub fn new(id: u16) -> Self {
        SpeciesId(id)
    }
}

/// The type of profession, includes NPCs and Gadgets
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize)]
pub enum Profession {
    Gadget,
    NonPlayableCharacter,
    Guardian,
    Warrior,
    Engineer,
    Ranger,
    Thief,
    Elementalist,
    Mesmer,
    Necromancer,
    Revenant,
    Dragonhunter,
    Berserker,
    Scrapper,
    Druid,
    Daredevil,
    Tempest,
    Chronomancer,
    Reaper,
    Herald,
    Soulbeast,
    Weaver,
    Holosmith,
    Deadeye,
    Mirage,
    Scourge,
    Spellbreaker,
    Firebrand,
    Renegade,
    Unknown,
}

impl Profession {
    pub fn core_profession(self) -> Profession {
        match self {
            Profession::Dragonhunter => Profession::Guardian,
            Profession::Firebrand    => Profession::Guardian,
            Profession::Berserker    => Profession::Warrior,
            Profession::Spellbreaker => Profession::Warrior,
            Profession::Herald       => Profession::Revenant,
            Profession::Renegade     => Profession::Revenant,
            Profession::Scrapper     => Profession::Engineer,
            Profession::Holosmith    => Profession::Engineer,
            Profession::Druid        => Profession::Ranger,
            Profession::Soulbeast    => Profession::Ranger,
            Profession::Daredevil    => Profession::Thief,
            Profession::Deadeye      => Profession::Thief,
            Profession::Tempest      => Profession::Elementalist,
            Profession::Weaver       => Profession::Elementalist,
            Profession::Chronomancer => Profession::Mesmer,
            Profession::Mirage       => Profession::Mesmer,
            Profession::Reaper       => Profession::Necromancer,
            Profession::Scourge      => Profession::Necromancer,
            x => x,
        }
    }
}

impl fmt::Display for Profession {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize)]
pub enum Boss {
    ValeGuardian,
    Gorseval,
    Sabetha,
    Slothasor,
    Matthias,
    KeepConstruct,
    Xera,
    Cairn,
    MursaatOverseer,
    Samarog,
    Deimos,
    SoullessHorror,
    Dhuum,
    Unknown,
}

impl Boss {
    pub fn from_species_id(species: SpeciesId) -> Boss {
        match species.0 {
            0x3c4e => Boss::ValeGuardian,
            0x3c45 => Boss::Gorseval,
            0x3c0f => Boss::Sabetha,
            0x3efb => Boss::Slothasor,
            0x3ef3 => Boss::Matthias,
            0x3f6b => Boss::KeepConstruct,
            0x3f76 => Boss::Xera,
            // ???
            // 0x3f9e => Boss::
            0x432a => Boss::Cairn,
            0x4314 => Boss::MursaatOverseer,
            0x4324 => Boss::Samarog,
            0x4302 => Boss::Deimos,
            0x4d37 => Boss::SoullessHorror,
            0x4bfa => Boss::Dhuum,
            _      => Boss::Unknown,
        }
    }
}