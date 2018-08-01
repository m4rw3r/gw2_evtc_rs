
const skills = [
  // Conditions
  { name: "Burning", id: 737, type: "Boon", boonStack: "Intensity", maxStacks: 1500, boonType: "Condition", icon: "https://wiki.guildwars2.com/images/thumb/4/45/Burning.png/20px-Burning.png" },
  { name: "Bleeding", id: 736, type: "Boon", boonStack: "Intensity", maxStacks: 1500, boonType: "Condition", icon: "https://wiki.guildwars2.com/images/thumb/3/33/Bleeding.png/20px-Bleeding.png" },
  { name: "Confusion", id: 861, type: "Boon", boonStack: "Intensity", maxStacks: 1500, boonType: "Condition", icon: "https://wiki.guildwars2.com/images/thumb/e/e6/Confusion.png/20px-Confusion.png" },
  { name: "Poison", id: 723, type: "Boon", boonStack: "Intensity", maxStacks: 1500, boonType: "Condition", icon: "https://wiki.guildwars2.com/images/thumb/0/05/Poison.png/20px-Poison.png" },
  { name: "Torment", id: 19426, type: "Boon", boonStack: "Intensity", maxStacks: 1500, boonType: "Condition", icon: "https://wiki.guildwars2.com/images/thumb/0/08/Torment.png/20px-Torment.png" },
  { name: "Blind", id: 720, type: "Boon", boonStack: "Duration", maxStacks: 9, boonType: "Condition", icon: "https://wiki.guildwars2.com/images/thumb/3/33/Blinded.png/20px-Blinded.png" },
  { name: "Chilled", id: 722, type: "Boon", boonStack: "Duration", maxStacks: 5,boonType: "Condition", icon: "https://wiki.guildwars2.com/images/thumb/a/a6/Chilled.png/20px-Chilled.png" },
  { name: "Crippled", id: 721, type: "Boon", boonStack: "Duration", maxStacks: 9, boonType: "Condition", icon: "https://wiki.guildwars2.com/images/thumb/f/fb/Crippled.png/20px-Crippled.png" },
  { name: "Fear", id: 791, type: "Boon", boonStack: "Duration", maxStacks: 9, boonType: "Condition", icon: "https://wiki.guildwars2.com/images/thumb/e/e6/Fear.png/20px-Fear.png" },
  { name: "Immobile", id: 727, type: "Boon", boonStack: "Duration", maxStacks: 3, boonType: "Condition", icon: "https://wiki.guildwars2.com/images/thumb/3/32/Immobile.png/20px-Immobile.png" },
  { name: "Slow", id: 26766, type: "Boon", boonStack: "Duration", maxStacks: 9, boonType: "Condition", icon: "https://wiki.guildwars2.com/images/thumb/f/fb/Slow_40px.png/20px-Slow_40px.png" },
  { name: "Weakness", id: 742, type: "Boon", boonStack: "Duration", maxStacks: 5, boonType: "Condition", icon: "https://wiki.guildwars2.com/images/thumb/f/f9/Weakness.png/20px-Weakness.png" },
  { name: "Taunt", id: 46996, type: "Boon", boonStack: "Duration", maxStacks: 5, boonType: "Condition", icon: "https://wiki.guildwars2.com/images/thumb/c/cc/Taunt.png/20px-Taunt.png" },
  { name: "Vulnerability", id: 738, type: "Boon", boonStack: "Intensity", maxStacks: 25, boonType: "Condition", icon: "https://wiki.guildwars2.com/images/thumb/a/af/Vulnerability.png/20px-Vulnerability.png" },
  { name: "Retaliation", id: 873, type: "Boon", boonStack: "Duration", maxStacks: 5, boonType: "Condition", icon: "https://wiki.guildwars2.com/images/5/53/Retaliation.png" },
  // Unlisted skills
  // Damage from One Wolf Pack
  { name: "One Wolf Pack", id: 42145, type: "Skill", icon: "https://render.guildwars2.com/file/4E137B09B5CC545211977BFDCEC76119195AC7C9/1770558.png" },
  { name: "Fiery Rush", id: 5663, type: "Skill", icon: "https://render.guildwars2.com/file/D7051607F7726AE1C3E4B80FE7F316244C075C0F/103329.png" },
  // Sigil effects
  { name: "Ring of Earth", id: 9433, type: "Sigil", icon: "https://wiki.guildwars2.com/images/4/43/Superior_Sigil_of_Geomancy.png" },
  { name: "Lightning Strike", id: 9292, type: "Sigil", icon: "https://wiki.guildwars2.com/images/c/c3/Superior_Sigil_of_Air.png" },
  // Pet skills
  { name: "Slash", id: 12655, type: "Skill", icon: "https://wiki.guildwars2.com/images/c/c3/Maul_%28feline%29.png" },
  { name: "Maul",  id: 12657, type: "Skill", icon: "https://wiki.guildwars2.com/images/c/c3/Maul_%28feline%29.png" },
  { name: "Bite",  id: 12694, type: "Skill", icon: "https://wiki.guildwars2.com/images/c/c2/Bite_%28feline%29.png" },
  // Clone skills
  { name: "Echo of Memory", id: 31864, type: "Skill", icon: "https://render.guildwars2.com/file/2B05E7099BC15D3A55C90D33AEB6939204DB92EA/1012889.png" },
  { name: "Mind Gash", id: 10298, type: "Skill", slot: "Weapon_1", icon: "https://render.guildwars2.com/file/CFFAD1180816A86DC03156B431A0B22C703FEAE4/103189.png" },
  { name: "Mind Slash", id: 10297, type: "Skill", slot: "Weapon_1", icon: "https://render.guildwars2.com/file/58635B4F6E0264FC59BC80B73706EFB7DE0E9A34/103188.png" },
  { name: "Mind Stab", id: 10299, type: "Skill", slot: "Weapon_1", icon: "https://render.guildwars2.com/file/58635B4F6E0264FC59BC80B73706EFB7DE0E9A34/103188.png" },
  { name: "Leap", id: 10228, type: "Skill", icon: "https://render.guildwars2.com/file/D7202F9A1D73AAF4D478B892BDEE017AAFA93EFB/103722.png" },
  { name: "Winds of Chaos", id: 10296, type: "Skill", icon: "https://render.guildwars2.com/file/0C9C043BFFC0773E390D19462444ABEB02FD4C01/103100.png" },
  { name: "Mage Strike", id: 10217, type: "Skill", icon: "https://render.guildwars2.com/file/095CB8FCE947F9D538CAD84839B475F2EEAC4A0C/103746.png" },
  { name: "Blurred Frenzy", id: 49067, type: "Skill", icon: "https://render.guildwars2.com/file/070633A302DA4865605316D1AF32DE40033CC0FE/103790.png" },
  { name: "Illusionary Sword Attack", id: 10230, type: "Skill", icon: "https://render.guildwars2.com/file/755CAC115104F0AA0630DCEB472D0678B62A916E/103723.png" },
  // Non-skills
  { name: "Dodge", id: 65001, type: "Dodge", icon: "https://wiki.guildwars2.com/images/c/cc/Dodge_Instructor.png" },
  { name: "Weapon Swap", id: 65534, type: "Weapon Swap", icon: "https://wiki.guildwars2.com/images/c/ce/Weapon_Swap_Button.png" },
  // TODO: More stuff
];
const skillData = {};
let   queue     = [];

// Default skills
skills.forEach(s => skillData[s.id] = Promise.resolve(s));

const requestIdleCb  = window.requestIdleCallback || window.setTimeout;
const GW2_API_SKILLS = "https://api.guildwars2.com/v2/skills?page_size=200&lang=en&ids=";

const fetchSkills = () => {
  const fetching = queue;

  queue = [];

  if( ! fetching.length) {
    return;
  }

  fetch(GW2_API_SKILLS + fetching.map(({ skillId }) => skillId).join(",")).then(resp => resp.json()).then(resp => {
    fetching.forEach(({ skillId, resolve, reject }) => {
      const skill = resp.find(s => s.id == skillId);

      if( ! skill) {
        reject(new Error(`api.guildwars2.com/v2/skills/${skillId}: Not Found`));
      }
      else {
        resolve(skill);
      }
    });
  });
}

export const getSkillData = skillId => {
  skillId = skillId|0;

  if( ! skillData[skillId]) {
    skillData[skillId] = new Promise((resolve, reject) => {
      queue.push({ skillId, resolve, reject });

      requestIdleCb(fetchSkills);
    });
  }

  return skillData[skillId];
}

export const groupBy = (items, getKey) => {
  const grouped = items.reduce((a, i) => {
    const k = getKey(i);

    a[k] = a[k] || [];
    a[k].push(i);

    return a;
  }, {});

  return Object.keys(grouped).sort().map(key => grouped[key]);
};