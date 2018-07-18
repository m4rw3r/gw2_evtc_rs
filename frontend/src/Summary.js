import { h
       , Component
       } from "preact";

import Profession from "./icons/Profession";
import { groupBy } from "./util";

export default class Summary extends Component {
  render({ encounter, players }, _, { duration, bossDuration }) {
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

    return <table class="summary">
      <tr>
        <th class="profession"></th>
        <th class="name">Name</th>
        <th class="subgroup">Group</th>
        <th class="dps">Boss DPS</th>
        <th class="dps-part">Power</th>
        <th class="dps-part">Condi</th>
        <th class="dps">All DPS</th>
        <th class="dps-part">Power</th>
        <th class="dps-part">Condi</th>
        <th class="scholar">Scholar</th>
        <th class="scholar-all">All</th>

      </tr>
      {sorted.map(Player)}
      {grouped.map(Group)}
      {Total(players)}
    </table>;
  }
}