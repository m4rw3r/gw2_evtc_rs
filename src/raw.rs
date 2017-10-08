use std::fmt;
use std::io;
use std::mem;
use std::slice;
use std::ffi::{CStr, FromBytesWithNulError};

fn buffer_to_cstr(buffer: &[u8]) -> Result<&CStr, FromBytesWithNulError> {
    CStr::from_bytes_with_nul(&buffer[..buffer.iter().position(|&c| c == 0).unwrap_or(0) + 1])
}

#[repr(packed)]
#[derive(Debug)]
pub struct Header {
    pub version:  [u8; 12],
    _pad1:        u8,
    pub boss_id:  u16,
    pub position: u8,
    pub actors:   u32,
}

#[repr(packed)]
pub struct Actor {
    pub id:          u64,
    pub proffession: u32,
    pub is_elite:    u32,
    pub toughness:   u32,
    pub healing:     u32,
    pub condition:   u32,
    name:            [u8; 68],
}

impl fmt::Debug for Actor {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Actor({}, {}, profession: {}, is_elite: {}, toughness: {}, healing: {}, condition: {})", self.id, self.name(), self.proffession, self.is_elite, self.toughness, self.healing, self.condition)
    }
}

impl Actor {
    pub fn name(&self) -> String {
        buffer_to_cstr(&self.name).expect("invalid C-string in EVTC Skill data")
            .to_str().expect("Invalid UTF8 in EVTC Skill data")
            .to_owned()
    }
}

#[repr(packed)]
pub struct Skill {
    pub id: u32,
    name:   [u8; 64],
}

impl fmt::Debug for Skill {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Skill({}, {})", self.id, self.name())
    }
}

impl Skill {
    pub fn name(&self) -> String {
        buffer_to_cstr(&self.name).expect("invalid C-string in EVTC Skill data")
            .to_str().expect("Invalid UTF8 in EVTC Skill data")
            .to_owned()
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum IFF {
    Friend  = 0,
    Foe     = 1,
    Unknown = 2,
}

#[repr(packed)]
#[derive(Debug, Clone, Copy)]
pub struct CombatEvent {
    pub time:              u64,
    pub src_agent:         u64,
    pub dst_agent:         u64,
    pub value:             i32,
    pub buff_dmg:          i32,
    pub overstack:         u16,
    pub skill_id:          u16,
    pub src_instid:        u16,
    pub dst_instid:        u16,
    pub src_master_instid: u16,
    _pad8:                 u64,
    _pad1:                 u8,
    pub iff:               IFF,
    buff:                  u8,
    pub result:            u8,
    pub is_activation:     u8,
    pub is_removal:        u8,
    pub is_ninety:         u8,
    pub is_fifty:          u8,
    pub is_moving:         u8,
    pub is_statechange:    u8,
    pub is_flanking:       u8,
    _pad2:                 u16,
    _pad1_1:               u8,
}

impl CombatEvent {
    pub fn is_buff(&self) -> bool {
        self.buff > 0
    }
}

#[derive(Debug)]
pub struct Evtc {
    pub header: Header,
    pub actors: Vec<Actor>,
    pub skills: Vec<Skill>,
    pub events: Vec<CombatEvent>,
}

fn read_struct<T, U: io::Read>(mut buffer: U) -> Result<T, io::Error> {
    let mut t: T = unsafe { mem::zeroed() };
    let size     = mem::size_of::<T>();

    unsafe {
        let t_slice = slice::from_raw_parts_mut(
            &mut t as *mut _ as *mut u8,
            size
        );

        // `read_exact()` comes from `Read` impl for `&[u8]`
        buffer.read_exact(t_slice)?;
    }

    Ok(t)
}

pub fn read<T: io::Read>(mut buffer: T) -> Result<Evtc, io::Error> {
    let header     = read_struct::<Header, _>(&mut buffer)?;
    let mut actors = Vec::with_capacity(header.actors as usize);

    for _ in 0..header.actors {
        actors.push(read_struct(&mut buffer)?);
    }

    let num_skills: u32 = read_struct(&mut buffer)?;
    let mut skills      = Vec::with_capacity(num_skills as usize);

    for _ in 0..num_skills {
        skills.push(read_struct(&mut buffer)?);
    }

    let mut events = Vec::new();
    let mut ev_buf: [u8; 64] = [0; 64];

    loop {
        match buffer.read_exact(&mut ev_buf) {
            Ok(_)  => events.push(read_struct(&ev_buf[..])?),
            Err(e) => match e.kind() {
               io::ErrorKind::UnexpectedEof => break,
               _                            => return Err(e),
            }
        }
    }

    Ok(Evtc {
        header: header,
        actors: actors,
        skills: skills,
        events: events,
    })
}
