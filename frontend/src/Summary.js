import { h
       , Component
       } from "preact";

import Profession from "./icons/Profession";
import Graph      from "./Graph";
import { groupBy } from "./util";

export default class Summary extends Component {
  render({ encounter, players, enemies }, _, { duration, bossDuration }) {
    const dps = ({ power, condi }, duration) => (power.totalDamage + condi.totalDamage) / duration;

    const Player        = ({ agent, bossHits, hits }) => <tr>
      <td class="profession"><Profession profession={agent.profession} /></td>
      <td class="name">{agent.name}</td>
      <td class="subgroup">{agent.subgroup}</td>
      <td class="dps">{dps(bossHits, bossDuration).toFixed(0)}</td>
      <td class="dps-part">{(bossHits.power.totalDamage / bossDuration).toFixed(0)}</td>
      <td class="dps-part">{(bossHits.condi.totalDamage / bossDuration).toFixed(0)}</td>
      <td class="dps">{dps(hits, duration).toFixed(0)}</td>
      <td class="dps-part">{(hits.power.totalDamage / duration).toFixed(0)}</td>
      <td class="dps-part">{(hits.condi.totalDamage / duration).toFixed(0)}</td>
      <td class="scholar">{(bossHits.power.scholar / bossHits.power.hits * 100).toFixed(2)}%</td>
      <td class="scholar-all">{(hits.power.scholar / hits.power.hits * 100).toFixed(2)}%</td>
    </tr>;
    const Group = () => null;
    const Total = () => null;

    const sorted  = players.slice().sort(({ bossHits: a }, { bossHits: b }) => dps(b, bossDuration) - dps(a, bossDuration))
    const grouped = groupBy(sorted, ({ agent: { subgroup }}) => subgroup);

    // return <pre>{JSON.stringify(sorted, null, 2)}</pre>;

    return <div class="summary">
        <Graph class="graph" series={[].concat.apply([], enemies.map(e => e.series))} width="1000" height="300" />
        <table>
        <tr>
          <th class="profession"></th>
          <th class="name">Name</th>
          <th class="subgroup">Group</th>
          <th class="dps" colspan="3">Boss DPS</th>
          <th class="dps" colspan="3">All DPS</th>
          <th class="scholar" colspan="2">Scholar</th>

        </tr>
        <tr>
          <th colspan="3"></th>
          <th class="dps-part">All</th>
          <th class="dps-part">Power</th>
          <th class="dps-part">Condi</th>
          <th class="dps-part">All</th>
          <th class="dps-part">Power</th>
          <th class="dps-part">Condi</th>
          <th class="scholar-all">Boss</th>
          <th class="scholar-all">All</th>
        </tr>
        {sorted.map(Player)}
        {grouped.map(Group)}
        {Total(players)}
      </table>
    </div>;
  }
}