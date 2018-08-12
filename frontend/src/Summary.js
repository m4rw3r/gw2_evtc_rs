import { h
       , cloneElement
       , Component
       } from "preact";
import { Link
       , Route
       , NavLink
       } from "react-router-dom";
import { parse as parseQueryString
       , stringify as stringifyQueryString
       } from "query-string";

import Profession      from "./icons/Profession";
import { downedIcon
       } from "./icons";
import { damageSeries
       , bossDmgSeries
       , fulltimeAvg
       , Axis
       , TimeAxis
       , Graph
       , HealthGraph
       , DPSGraph
       , GroupedGraphs
       , ResponsiveGraph
       } from "./Graph";
import { groupBy
       , contextData
       , professionColour
       } from "./util";
import { buffIcons
       } from "./generatedData";

const agentName      = ({ agent: { name: a } })                 => a;
const agentSubgroup  = ({ agent: { subgroup: a } })             => a;
const bossHits       = ({ bossHits: a })                        => a;
const hits           = ({ hits: a })                            => a;
const powerDamage    = ({ power: { totalDamage: a } })          => a;
const condiDamage    = ({ condi: { totalDamage: a } })          => a;
const incomingDamage = ({ incomingDamage: { total: { totalDamage: a } } }) => a;
const scholarUptime  = ({ power: { scholar, hits } })           => scholar / hits;
const critRate       = ({ power: { criticals, hits } })         => criticals / hits;
const totalDamage    = ({ power: { totalDamage: a }, condi: { totalDamage: b } }) => a + b;
const downed         = ({ series }) => series.filter(e => e.downed);
const wastedSkills   = ({ activationLog }) => activationLog.filter(e => e.canceled);
const wastedTime     = (player)            => wastedSkills(player).reduce((a, e) => a + e.duration, 0);

const SORT_FUNCS = {
  name: (a, b) => agentName(a).localeCompare(agentName(b)),
  group: (a, b) => agentSubgroup(a).localeCompare(agentSubgroup(b)),
  bossDps: (a, b) => totalDamage(bossHits(b)) - totalDamage(bossHits(a)),
  bossPowerDps: (a, b) => powerDamage(bossHits(b)) - powerDamage(bossHits(a)),
  bossCondiDps: (a, b) => condiDamage(bossHits(b)) - condiDamage(bossHits(a)),
  dps: (a, b) => totalDamage(hits(b)) - totalDamage(hits(a)),
  powerDps: (a, b) => powerDamage(hits(b)) - powerDamage(hits(a)),
  condiDps: (a, b) => condiDamage(hits(b)) - condiDamage(hits(a)),
  incoming: (a, b) => incomingDamage(b) - incomingDamage(a),
  wasted: (a, b) => wastedTime(b) - wastedTime(a),
  critBoss: (a, b) => critRate(bossHits(b)) - critRate(bossHits(a)),
  crit: (a, b) => critRate(hits(b)) - critRate(hits(a)),
  scholar: (a, b) => scholarUptime(hits(b)) - scholarUptime(hits(a)),
  scholarBoss: (a, b) => scholarUptime(bossHits(b)) - scholarUptime(bossHits(a)),
  downed: (a, b) => downed(b).length - downed(a).length,
};

const boonSort = skillId => ({ buffs: a }, { buffs: b }) => (b[skillId] || EMPTY_BUFF).uptime - (a[skillId] || EMPTY_BUFF).uptime;

const sortFromParams = ({ sort="name", reverse=false, boon }) => {
  const sortFn = sort === "boon" ? boonSort(boon) : (SORT_FUNCS[sort] || SORT_FUNCS.name);

  if(reverse) {
    return (a, b) => sortFn(b, a);
  }
  
  return sortFn;
}

const TH = ({ sort, boon, children, ...rest }) => <Route render={({ location: { pathname, search }}) => {
  const { sort: oldSort="name", reverse: oldReverse=false, boon: oldBoon } = parseQueryString(search);
  const match = oldSort === sort && (oldBoon|0) === (boon|0);
  const newSearch = stringifyQueryString({
    sort:    sort !== "name" ? sort : undefined,
    reverse: match && !oldReverse ? true : undefined,
    boon:    boon || undefined,
  });

  return <th className={`sortable${match ? " active" : ""}`} {...rest}>
    <Link to={{
      pathname,
      search: newSearch,
    }}>
      <span>
      {children}
      </span>
    </Link>
  </th>;
}} />

const seconds = time => (time / 1000).toFixed(2);

class PlayerDamageRow extends Component {
  render({ agent: { name, profession, subgroup }, bossHits, hits, incomingDamage: { total: { totalDamage: incomingDamage } }, activationLog, series }, _, { format: { bossDps, dps, percent, damage, number } }) {
    return <tr>
      <td class="icon" title={profession}><Profession profession={profession} /></td>
      <td class="name">{name}</td>
      <td class="subgroup">{subgroup}</td>
      <td class="number" title={damage(totalDamage(bossHits)) + " dmg"}>{bossDps(totalDamage(bossHits))}</td>
      <td class="number secondary" title={damage(powerDamage(bossHits)) + " dmg"}>{bossDps(powerDamage(bossHits))}</td>
      <td class="number secondary" title={damage(condiDamage(bossHits)) + " dmg"}>{bossDps(condiDamage(bossHits))}</td>
      <td class="number" title={damage(totalDamage(hits)) + " dmg"}>{dps(totalDamage(hits))}</td>
      <td class="number secondary" title={damage(powerDamage(hits)) + " dmg"}>{dps(powerDamage(hits))}</td>
      <td class="number secondary" title={damage(condiDamage(hits)) + " dmg"}>{dps(condiDamage(hits))}</td>
      <td class="number" title={dps(incomingDamage) + " dps"}>{damage(incomingDamage)}</td>
      <td class="number" title={`${wastedSkills({ activationLog }).length} canceled skills`}>{seconds(wastedTime({ activationLog }))}</td>
      <td class="number" title={`${bossHits.power.criticals} / ${bossHits.power.hits}`}>{percent(critRate(bossHits), 1)}</td>
      <td class="number secondary" title={`${hits.power.criticals} / ${hits.power.hits}`}>{percent(critRate(hits), 1)}</td>
      <td class="number" title={`${bossHits.power.scholar} / ${bossHits.power.hits}`}>{percent(scholarUptime(bossHits), 1)}</td>
      <td class="number secondary" title={`${hits.power.scholar} / ${hits.power.hits}`}>{percent(scholarUptime(hits), 1)}</td>
      <td class="downed">{number(downed({ series }).length)}</td>
    </tr>;
  }
}

@contextData(({ location: { search } }, { players }) => ({ players, sort: sortFromParams(parseQueryString(search)) }))
export class DamageTable extends Component {
  render({ players, sort }, _, { format: { bossDps, dps, damage } }) {
    const groupTotal = agents => {
      const totalBossDamage = agents.reduce((b, a) => totalDamage(a.bossHits) + b, 0);
      const powerBossDamage = agents.reduce((b, a) => a.bossHits.power.totalDamage + b, 0);
      const condiBossDamage = agents.reduce((b, a) => a.bossHits.condi.totalDamage + b, 0);
      const totalAllDamage = agents.reduce((b, a) => totalDamage(a.hits) + b, 0);
      const powerAllDamage = agents.reduce((b, a) => a.hits.power.totalDamage + b, 0);
      const condiAllDamage = agents.reduce((b, a) => a.hits.condi.totalDamage + b, 0);

      return [
        <td class="number" title={damage(totalBossDamage) + " dmg"}>{bossDps(totalBossDamage)}</td>,
        <td class="number secondary" title={damage(powerBossDamage) + " dmg"}>{bossDps(powerBossDamage)}</td>,
        <td class="number secondary" title={damage(condiBossDamage) + " dmg"}>{bossDps(condiBossDamage)}</td>,
        <td class="number" title={damage(totalAllDamage) + " dmg"}>{dps(totalAllDamage)}</td>,
        <td class="number secondary" title={damage(powerAllDamage) + " dmg"}>{dps(powerAllDamage)}</td>,
        <td class="number secondary" title={damage(condiAllDamage) + " dmg"}>{dps(condiAllDamage)}</td>,
      ];
    }

    const GroupDamageRow = (agents) => <tr class="group-total">
      <td></td>
      <td>Group {agents[0].agent.subgroup}</td>
      <td></td>
      {groupTotal(agents)}
      <td></td>
      <td></td>
      <td></td>
      <td></td>
      <td></td>
      <td></td>
      <td></td>
    </tr>;
    const TotalDamageRow = agents => <tr class="total">
      <td></td>
      <td>Total</td>
      <td></td>
      {groupTotal(agents)}
      <td></td>
      <td></td>
      <td></td>
      <td></td>
      <td></td>
      <td></td>
      <td></td>
    </tr>;

    const sorted  = players.slice().sort(sort);
    const grouped = groupBy(sorted, ({ agent: { subgroup }}) => subgroup);

    return <table>
      <tr>
        <th></th>
        <TH sort="name" title="Character name">Name</TH>
        <TH sort="group" title="Subgroup">Group</TH>
        <TH sort="bossDps" colspan="3">Boss DPS</TH>
        <TH sort="dps" colspan="3">All DPS</TH>
        <TH sort="incoming" title="Incoming damage">Incoming</TH>
        <TH sort="wasted" title="Wasted time casting skills which got canceled">Wasted</TH>
        <TH sort="critBoss" colspan="2" title="Percentage of hits which were critical hits">Crits</TH>
        <TH sort="scholarBoss" colspan="2" title="Percentage of hits which potentially benefited from the >90% Scholar rune bonus">Scholar</TH>
        <TH sort="downed" title="Number of times player got downed."><img alt="Downed" src={downedIcon} /></TH>
      </tr>
      <tr class="subheading">
        <th colspan="3"></th>
        <TH sort="bossDps">All</TH>
        <TH sort="bossPowerDps">Power</TH>
        <TH sort="bossCondiDps">Condi</TH>
        <TH sort="dps">All</TH>
        <TH sort="powerDps">Power</TH>
        <TH sort="condiDps">Condi</TH>
        <th></th>
        <th></th>
        <TH sort="critBoss">Boss</TH>
        <TH sort="crit">All</TH>
        <TH sort="scholarBoss">Boss</TH>
        <TH sort="scholar">All</TH>
        <th></th>
      </tr>
      {sorted.map(p => <PlayerDamageRow {...p} />)}
      {grouped.map(GroupDamageRow)}
      {TotalDamageRow(players)}
    </table>;
  }
}

const EMPTY_BUFF = {
  overstack: 0,
  stacks:    0,
  stripped:  0,
  sum:       0,
  uptime:    0,
};

class PlayerBoonRow extends Component {
  render({ agent: { name, profession, subgroup }, series, buffs, buffOrder }, _, { encounter: { duration }, format: { percent, number } }) {
    const BuffRow = ({ skillId, stack: { type, max: maxStacks } }) => {
      const { uptime, overstack } = buffs[skillId] || EMPTY_BUFF;

      switch(type) {
      case "Duration":
        return <td class="number" title={`Including overstack: ${(uptime + overstack) / duration}`}>{percent(uptime / duration, 0)}</td>;
      case "Intensity":
        return <td class="number" title={`Including overstack: ${(uptime + overstack) / duration}`}>{number(uptime / duration, 1)}</td>;
      }
    };

    return <tr>
      <td class="icon" title={profession}><Profession profession={profession} /></td>
      <td class="name">{name}</td>
      <td class="subgroup">{subgroup}</td>
      {buffOrder.map(BuffRow)}
    </tr>;
  }
}

const buffSort = ({ name: a }, { name: b }) => a.localeCompare(b);

@contextData(({ location: { search } }, { players, buffs }) => ({ buffs, players, sort: sortFromParams(parseQueryString(search)) }))
export class BoonTable extends Component {
  render({ sort, buffs, players }) {
    const sorted  = players.slice().sort(sort);
    const grouped = groupBy(sorted, ({ agent: { subgroup }}) => subgroup);

    const buffsSorted = Object.values(buffs).sort(buffSort);
    const BuffHeading = ({ name, skillId }) => <TH sort="boon" boon={skillId} title={name}>{
      buffIcons[name] ? <img alt={name} src={buffIcons[name]} /> : name
    }</TH>;

    return <table>
      <tr>
        <th></th>
        <TH sort="name" title="Character name">Name</TH>
        <TH sort="group" title="Subgroup">Group</TH>
        {buffsSorted.map(BuffHeading)}
      </tr>
      {sorted.map(p => <PlayerBoonRow {...p} buffOrder={buffsSorted} />)}
    </table>;
  }
}

@contextData(({ location: { search } }, { encounter, enemies, players }) => ({ encounter, enemies, players, search }))
export class Summary extends Component {
  render({ encounter, players, enemies, search, children }, _, { format: { damage } }) {
    return <div class="summary">
      <ResponsiveGraph class="graph">
        <Graph start={encounter.seriesStart / 1000} end={encounter.seriesEnd / 1000} width="1500" height="300">
          <GroupedGraphs>
            {/*players.map(p => <DPSGraph class="line" series={damageSeries(p.series)} />)*/}
            {players.map(p => <DPSGraph class="line" style={{ stroke: professionColour(p.agent) }} series={fulltimeAvg(bossDmgSeries(p.series), encounter.seriesStart / 1000)} />)}
            <Axis format={damage} class="damage-axis" />
          </GroupedGraphs>
          <HealthGraph class="line" style={{ stroke: "#FFCC66", "stroke-dasharray": 5 }} series={[].concat.apply([], enemies.map(e => e.series))} />
          <TimeAxis class="time-axis" />
        </Graph>
      </ResponsiveGraph>

      <ul class="section-list">
        <li><NavLink exact to={{ pathname: "/", search }}>Damage</NavLink></li>
        <li><NavLink to={{ pathname: "/boons", search }}>Boons</NavLink></li>
        <li><NavLink to={{ pathname: "/mechanics", search }}>Mechanics</NavLink></li>
      </ul>

      {children}
    </div>;
  }
}