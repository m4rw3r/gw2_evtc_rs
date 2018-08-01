import { h
       , Component
       } from "preact";

import Profession from "./icons/Profession";
import { getSkillData } from "./util";

class Agent extends Component {
  render({ agent, bossHits: { abilities }, abilityNames, skillData, allBossDamage }, _, { format: { dps, damage, number, percent } }) {
    const list = Object.keys(abilities).map(k => ({...abilities[k], name: abilityNames[k], key: k, skillData: skillData[k|0] })).sort((a, b) => b.totalDamage - a.totalDamage);

    const Ability = ({ name, key, totalDamage, hits, criticals, flanking, glancing, scholar, moving, interrupted, blocked, evaded, absorbed, missed, minDamage, maxDamage, skillData }) => <tr>
      <td class="icon">{skillData ? <img src={skillData.icon} /> : null}</td>
      <td class="name" title={key}>{name || (skillData && skillData.name) || key}</td>
      <td class="number">{damage(totalDamage)}</td>
      <td class="number secondary">{percent(totalDamage / allBossDamage)}</td>
      <td class="number">{number(hits)}</td>
      <td class="number">{number(criticals)}</td>
      <td class="number secondary">{number(scholar)}</td>
      <td class="number secondary">{number(flanking)}</td>
      <td class="number secondary">{number(glancing)}</td>
      <td class="number secondary">{number(moving)}</td>
      <td class="number secondary">{number(blocked + evaded + absorbed + missed + interrupted)}</td>
      <td class="number">{damage(minDamage)}</td>
      <td class="number">{damage(maxDamage)}</td>
      <td class="number">{damage(Math.round(totalDamage / hits))}</td>
    </tr>;

    return <div class="agent">
      <table class="ability-list">
        <tr>
          <th colspan="2">Skill</th>
          <th colspan="2">Total Damage</th>
          <th colspan="7">Hits</th>
          <th colspan="3">Damage</th>
        </tr>
        <tr class="subheading">
          <th colspan="2"></th>
          <th></th>
          <th>%</th>
          <th></th>
          <th>Criticals</th>
          <th>Scholar</th>
          <th>Flanking</th>
          <th>Glancing</th>
          <th>Moving</th>
          <th title="Also includes blocked, missed, evaded hits">Absorbed</th>
          <th>Min</th>
          <th>Max</th>
          <th>Avg</th>
        </tr>
        {list.map(Ability)}
      </table>
    </div>
  }
}

export default class PlayerSummary extends Component {
  constructor() {
    super();

    this.state = {
      selectedAgent: null,
      skillData:     {},
    };
  }
  componentWillMount() {
    this.loadSkills(this.props.player);
  }
  loadSkills(player) {
    // Load all skill-icons
    const skillIds = [].concat.apply([], player.agents.map(({ bossHits: { abilities }}) => Object.keys(abilities)))
        .concat(player.activationLog.map(({ skill }) => skill))
        .filter((v, i, a) => a.indexOf(v) === i);

    skillIds.map(getSkillData).forEach(p => p.then(s => this.setState({
      skillData: {...this.state.skillData, [s.id]: s},
    })));
  }
  render({ player, skills }, { skillData, selectedAgent }, { format: { time: formatTime } }) {
    if( ! player) {
      return <div>No player found</div>;
    }

    const { agent: { name, profession }, bossHits, agents, activationLog } = player
    const currentAgent = agents.find(a => a.agent.speciesId === selectedAgent);

    const CastSkill = ({ time, skill, quickness, canceled, duration }, timeDiff) => {
      const d = skillData[skill|0];

      if( ! d) {
        return;
      }

      if(d.slot === "Weapon_1") {
        return null;
      }

      let classNames = ["activation"];

      if(canceled) {
        classNames.push("canceled");
      }

      if(quickness) {
        classNames.push("quickness");
      }


      return <li class={classNames.join(" ")} title={formatTime(time) + " " + d.name}
        style={`flex-basis: ${timeDiff / 200}%`}>
        <img src={skillData[skill|0].icon}/>
      </li>;
    }

    const log      = [];
    let   logStart = activationLog.length > 0 ? activationLog[0].time : 0;

    for(let i = 0; i < activationLog.length; i++) {
      const diff = i + 1 < activationLog.length ? activationLog[i + 1].time - activationLog[i].time: 0;

      log.push(CastSkill(activationLog[i], diff));
    }

    return <div class="player-summary">
      <h3>
        <Profession class="profession" profession={profession} />

        <span>{name}</span>
      </h3>

      {agents.length > 1 ?
      <ul class="agent-selection">
        {agents.map(({ agent }) => <li onClick={() => this.setState({ selectedAgent: agent.speciesId })}
            class={agent.speciesId === selectedAgent ? "selected" : null }>
            {agent.profession !== "NonPlayableCharacter" ? <Profession class="agent-profession" profession={agent.profession} /> : null }{agent.name}
          </li>)}
      </ul> : null}

      <Agent {...currentAgent} abilityNames={skills} skillData={skillData} allBossDamage={bossHits.condi.totalDamage + bossHits.power.totalDamage} />

      <h3>
        Skill Rotation
      </h3>

      <ul class="activations">
        {log}
      </ul>
    </div>;
  }
}