
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