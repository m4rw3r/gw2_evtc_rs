import { h
       , Component
       } from "preact";

import Profession from "./icons/Profession";
import Graph      from "./Graph";
import { groupBy } from "./util";

const totalDamage = ({ power, condi }) => power.totalDamage + condi.totalDamage;
const reverseSort = (func) => {
  let reversed = (a, b) => func(b, a);

  reversed.func = func;

  return reversed;
};
const nameSort     = ({ agent: { name: a } }, { agent: { name: b } }) => a.localeCompare(b);
const groupSort    = ({ agent: { subgroup: a } }, { agent: { subgroup: b } }) => a.localeCompare(b);
const bossDpsSort  = ({ bossHits: a }, { bossHits: b }) => totalDamage(b) - totalDamage(a);
const dpsSort      = ({ hits: a }, { hits: b }) => totalDamage(b) - totalDamage(a);
const incomingSort = ({ incomingDamage: { totalDamage: a } }, { incomingDamage: { totalDamage: b } }) => b - a;
const scholarSort  = ({ hits: { power: { scholar: aScholar, hits: aHits } } }, { hits: { power: { scholar: bScholar, hits: bHits } } }) => (bScholar / bHits) - (aScholar / aHits);

export default class Summary extends Component {
  constructor(props) {
    super(props);

    this.state = {
      sort: bossDpsSort,
    };
  }

  setSort(func) {
    if(this.state.sort === func) {
      this.setState({
        sort: reverseSort(func),
      });
    }
    else {
      this.setState({
        sort: func,
      });
    }
  }

  render({ encounter, players, enemies }, { sort }, { encounter: { duration }, boss: { duration: bossDuration }, format: { dps, percent, damage, number } }) {
    const Player = ({ agent, bossHits, hits }) => <tr>
      <td class="icon"><Profession profession={agent.profession} /></td>
      <td class="name">{agent.name}</td>
      <td class="subgroup">{agent.subgroup}</td>
      <td class="number" title={damage(totalDamage(bossHits)) + " dmg"}>{dps(totalDamage(bossHits))}</td>
      <td class="number secondary" title={damage(bossHits.power.totalDamage) + " dmg"}>{dps(bossHits.power.totalDamage)}</td>
      <td class="number secondary" title={damage(bossHits.condi.totalDamage) + " dmg"}>{dps(bossHits.condi.totalDamage)}</td>
      <td class="number" title={damage(totalDamage(hits)) + " dmg"}>{dps(totalDamage(hits))}</td>
      <td class="number secondary" title={damage(hits.power.totalDamage) + " dmg"}>{dps(hits.power.totalDamage)}</td>
      <td class="number secondary" title={damage(hits.condi.totalDamage) + " dmg"}>{dps(hits.condi.totalDamage)}</td>
      <td class="number" title={`${bossHits.power.scholar} / ${bossHits.power.hits}`}>{percent(bossHits.power.scholar / bossHits.power.hits)}</td>
      <td class="number secondary" title={`${hits.power.scholar} / ${hits.power.hits}`}>{percent(hits.power.scholar / hits.power.hits)}</td>
    </tr>;

    const groupTotal = agents => {
      const totalBossDamage = agents.reduce((b, a) => totalDamage(a.bossHits) + b, 0);
      const powerBossDamage = agents.reduce((b, a) => a.bossHits.power.totalDamage + b, 0);
      const condiBossDamage = agents.reduce((b, a) => a.bossHits.condi.totalDamage + b, 0);
      const totalAllDamage = agents.reduce((b, a) => totalDamage(a.hits) + b, 0);
      const powerAllDamage = agents.reduce((b, a) => a.hits.power.totalDamage + b, 0);
      const condiAllDamage = agents.reduce((b, a) => a.hits.condi.totalDamage + b, 0);

      return [
        <td class="number" title={damage(totalBossDamage) + " dmg"}>{dps(totalBossDamage)}</td>,
        <td class="number secondary" title={damage(powerBossDamage) + " dmg"}>{dps(powerBossDamage)}</td>,
        <td class="number secondary" title={damage(condiBossDamage) + " dmg"}>{dps(condiBossDamage)}</td>,
        <td class="number" title={damage(totalAllDamage) + " dmg"}>{dps(totalAllDamage)}</td>,
        <td class="number secondary" title={damage(powerAllDamage) + " dmg"}>{dps(powerAllDamage)}</td>,
        <td class="number secondary" title={damage(condiAllDamage) + " dmg"}>{dps(condiAllDamage)}</td>,
      ];
    }

    const Group = (agents) => <tr class="group-total">
      <td></td>
      <td>Group {agents[0].agent.subgroup}</td>
      <td></td>
      {groupTotal(agents)}
      <td></td>
      <td></td>
    </tr>;
    const Total = agents => <tr class="total">
      <td></td>
      <td>Total</td>
      <td></td>
      {groupTotal(agents)}
      <td></td>
      <td></td>
    </tr>;

    const sorted  = players.slice().sort(sort);
    const grouped = groupBy(sorted, ({ agent: { subgroup }}) => subgroup);

    return <div class="summary">
        <Graph class="graph" series={[].concat.apply([], enemies.map(e => e.series))} width="1000" height="300" />
        <table>
        <tr>
          <th></th>
          <th class={[sort, sort.func].indexOf(nameSort) !== -1    ? "selected" : ""} onClick={() => this.setSort(nameSort)}>Name</th>
          <th class={[sort, sort.func].indexOf(groupSort) !== -1   ? "selected" : ""} onClick={() => this.setSort(groupSort)}>Group</th>
          <th class={[sort, sort.func].indexOf(bossDpsSort) !== -1 ? "selected" : ""} colspan="3" onClick={() => this.setSort(bossDpsSort)}>Boss DPS</th>
          <th class={[sort, sort.func].indexOf(dpsSort) !== -1     ? "selected" : ""} colspan="3" onClick={() => this.setSort(dpsSort)}>All DPS</th>
          <th class={[sort, sort.func].indexOf(scholarSort) !== -1 ? "selected" : ""} colspan="2" onClick={() => this.setSort(scholarSort)}>Scholar</th>
        </tr>
        <tr class="subheading">
          <th colspan="3"></th>
          <th class={[sort, sort.func].indexOf(bossDpsSort) !== -1 ? "selected" : ""} onClick={() => this.setSort(bossDpsSort)}>All</th>
          <th>Power</th>
          <th>Condi</th>
          <th class={[sort, sort.func].indexOf(dpsSort) !== -1 ? "selected" : ""} onClick={() => this.setSort(dpsSort)}>All</th>
          <th>Power</th>
          <th>Condi</th>
          <th class={[sort, sort.func].indexOf(scholarSort) !== -1 ? "selected" : ""} onClick={() => this.setSort(scholarSort)}>Boss</th>
          <th>All</th>
        </tr>
        {sorted.map(Player)}
        {grouped.map(Group)}
        {Total(players)}
      </table>
    </div>;
  }
}