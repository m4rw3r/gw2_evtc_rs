import { h
       , Component
       } from "preact";

import Profession from "./icons/Profession";
import { getSkillData } from "./util";

const Agent = ({ agent, bossHits: { abilities, total }, abilityNames, skillData }) => {
  const list = Object.keys(abilities).map(k => ({...abilities[k], name: abilityNames[k], key: k, skillData: skillData[k|0] })).sort((a, b) => b.totalDamage - a.totalDamage);

  const Ability = ({ name, key, totalDamage, hits, criticals, flanking, glancing, scholar, moving, interrupted, blocked, evaded, absorbed, missed, minDamage, maxDamage, skillData }) => <tr>
    <td>{skillData ? <img src={skillData.icon} /> : null}</td>
    <td class="name">{name || (skillData && skillData.name) || key}</td>
    <td>{totalDamage}</td>
    <td>{(totalDamage / total.totalDamage * 100).toFixed(2)}%</td>
    <td>{hits}</td>
    <td>{criticals}</td>
    <td>{scholar}</td>
    <td>{flanking}</td>
    <td>{glancing}</td>
    <td>{moving}</td>
    <td>{interrupted}</td>
    <td>{blocked + evaded + absorbed + missed}</td>
    <td>{minDamage}</td>
    <td>{maxDamage}</td>
  </tr>;

  return <div class="agent">
    <table class="ability-list">
      <tr>
        <th></th>
        <th class="name">Skill name</th>
        <th>Total Damage</th>
        <th>(%)</th>
        <th>Hits</th>
        <th>Criticals</th>
        <th>Scholar</th>
        <th>Flanking</th>
        <th>Glancing</th>
        <th>Moving</th>
        <th>Interrupted</th>
        <th title="Also includes blocked, missed, evaded hits">Absorbed</th>
        <th>Min Damage</th>
        <th>Max Damage</th>
      </tr>
      {list.map(Ability)}
    </table>
  </div>
};

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
        .filter((v, i, a) => a.indexOf(v) === i);

    skillIds.map(getSkillData).forEach(p => p.then(s => this.setState({
      skillData: {...this.state.skillData, [s.id]: s},
    })));
  }
  componentDidUpdate(prebProps) {
    if(prebProps.player !== this.props.player) {
      this.loadSkills(this.props.player);

      this.setState({
        selectedAgent: null
      });
    }
  }
  render({ player, skills }, { skillData, selectedAgent }) {
    if( ! player) {
      return <div>No player found</div>;
    }

    const { agent: { name, profession }, agents } = player
    const currentAgent = agents.find(a => a.agent.speciesId === selectedAgent);

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

      <Agent {...currentAgent} abilityNames={skills} skillData={skillData} />

      <pre>
        {JSON.stringify(currentAgent, null, 2)}
      </pre>
    </div>;
  }
}