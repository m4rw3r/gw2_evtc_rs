import { h
       , Component
       } from "preact";
import { buffIcons
       , skillIcons as generatedSkillIcons
       } from "./generatedData";

export const professionColour = ({ profession }) => {
  switch(profession) {
  case "Dragonhunter":
  case "Firebrand":
  case "Guardian":
    return "#72C1D9";
  case "Revenant":
  case "Herald":
  case "Renegade":
    return "#D16E5A";
  case "Warrior":
  case "Spellbreaker":
  case "Berserker":
    return "#FFD166";
  case "Engineer":
  case "Scrapper":
  case "Holosmith":
    return "#D09C59";
  case "Ranger":
  case "Druid":
  case "Soulbeast":
    return "#8CDC82";
  case "Thief":
  case "Daredevil":
  case "Deadeye":
    return "#C08F95";
  case "Elementalist":
  case "Tempest":
  case "Weaver":
    return "#F68A87";
  case "Mesmer":
  case "Chronomancer":
  case "Mirage":
    return "#B679D5";
  case "Necromancer":
  case "Reaper":
  case "Scourge":
    return "#52A76F";
  default:
    return "#BBBBBB";
  }
}

type Skill = {
  name: string,
  id:   number,
  icon: string,
};

const skills: Array<Skill> = [
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
export const skillIcons = skills.reduce((a, s) => {
  if( ! a[s.id]) {
    a[s.id] = s.icon;
  }

  return a;
}, generatedSkillIcons);

export const SkillIcon = ({ name, skillId }) => skillIcons[skillId]
  ? <img alt={name} src={skillIcons[skillId]} />
  : buffIcons[name]
    ? <img alt={name} src={buffIcons[name]} />
    : skillId;

export const groupBy = (items, getKey) => {
  const grouped = items.reduce((a, i) => {
    const k = getKey(i);

    a[k] = a[k] || [];
    a[k].push(i);

    return a;
  }, {});

  return Object.keys(grouped).sort().map(key => grouped[key]);
};

export function contextData<P, Q, C>(fn: (props: P, context: C) => Q): (component: Component<Q, C>) => Component<P, C> {
  return (component: Context<Q, C>) => class ContextData extends Component {
    render(props: P, _, ctx: C) {
      return h(component, fn(props, ctx), ...props.children);
    }
  }
}