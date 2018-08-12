import { h
       , cloneElement
       , Component
       } from "preact";
import { NavLink
       , Route } from "react-router-dom";

import Profession from "./icons/Profession";
import { contextData
       , skillIcons
       , SkillIcon
       } from "./util";
import { weapon1Skills
       , skillNames
       } from "./generatedData";

@contextData(({ match: { params: { speciesId=null } }, player }, { skills }) => ({
  skills,
  player,
  agentData: player.agents.find(a => (a.agent.speciesId|0) === (speciesId|0))
}))
export class AgentSummary extends Component {
  render({ agentData, player, skills }, _, { format: { dps, damage, number, percent } }) {
    if( ! agentData) {
      return <p>Not found</p>;
    }

    const { bossHits: { condi, power } } = player;
    const { agent, bossHits: { abilities } }    = agentData;
    const allBossDamage = condi.totalDamage + power.totalDamage;
    const list = Object.keys(abilities).map(k => ({...abilities[k], name: skills[k], skillId: k|0 })).sort((a, b) => b.totalDamage - a.totalDamage);

    const Ability = ({ name, skillId, totalDamage, hits, criticals, flanking, glancing, scholar, moving, interrupted, blocked, evaded, absorbed, missed, minDamage, maxDamage }) => <tr>
      <td class="icon"><SkillIcon skillId={skillId} name={name || skillId} /></td>
      <td class="name" title={skillId}>{name || skillId}</td>
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

const CastSkill = ({ time, skill, quickness, canceled, duration }, timeDiff, formatTime) => {
  if( ! skillIcons[skill|0]) {
    return;
  }

  // TODO: Add global checkbox for this setting
  if(weapon1Skills[skill|0]) {
    return null;
  }

  let classNames = ["activation"];

  if(canceled) {
    classNames.push("canceled");
  }

  if(quickness) {
    classNames.push("quickness");
  }

  return <li class={classNames.join(" ")} title={formatTime(time) + " " + skillNames[skill]}
    style={`flex-basis: ${timeDiff / 200}%`}>
    <SkillIcon skillId={skill} name={skillNames[skill]} />
  </li>;
}

export class ActivationLog extends Component {
  render({ player: { activationLog } }, _, { format: { time }}) {
    const log      = [];
    let   logStart = activationLog.length > 0 ? activationLog[0].time : 0;

    for(let i = 0; i < activationLog.length; i++) {
      const diff = i + 1 < activationLog.length ? activationLog[i + 1].time - activationLog[i].time: 0;

      log.push(CastSkill(activationLog[i], diff, time));
    }

    return <div>
      <h3>
        Skill Rotation
      </h3>

      <ul class="activations">
        {log}
      </ul>
    </div>;
  }
}

const sortAgents = ({ agent: { name: aName, speciesId: aSpecies }},
                    { agent: { name: bName, speciesId: bSpecies }}) =>
  (Boolean(aSpecies) ^ Boolean(bSpecies)) ? (aSpecies|0) - (bSpecies|0) : aName.localeCompare(bName);

@contextData(({ match: { params: { name } } }, { players, skills }) => ({
  player: players.find(p => p.agent.name === name),
  skills,
}))
export class PlayerSummary extends Component {
  render({ player, skills, children }, _, { format: { time: formatTime } }) {
    if( ! player) {
      return <div>No player found</div>;
    }

    const { agent: { name, profession }, agents } = player

    return <div class="player-summary">
      <h3>
        <Profession class="profession" profession={profession} />

        <span>{name}</span>
      </h3>

      {agents.length > 1 ?
      <ul class="agent-selection">
        {agents.slice().sort(sortAgents).map(({ agent }) => <li>
          <NavLink exact to={agent.speciesId ? `/players/${name}/agents/${agent.speciesId}` : `/players/${name}`}>
            {agent.profession !== "NonPlayableCharacter" ? <Profession class="agent-profession" profession={agent.profession} /> : null }{agent.name}
          </NavLink>
        </li>)}
      </ul> : null}

      {children.map(c => cloneElement(c, { player }))}
    </div>;
  }
}