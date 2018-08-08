import { h
       , cloneElement
       , Component
       } from "preact";

import Profession      from "./icons/Profession";
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
import { groupBy }     from "./util";

const DAMAGE    = "section_damage";
const BOONS     = "section_boons";
const MECHANICS = "section_mechanics";

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

const reverseSort = (func) => {
  let reversed = (a, b) => func(b, a);

  reversed.func = func;

  return reversed;
};
const nameSort        = (a, b) => agentName(a).localeCompare(agentName(b));
const groupSort       = (a, b) => agentSubgroup(a).localeCompare(agentSubgroup(b));
const bossDpsSort     = (a, b) => totalDamage(bossHits(b)) - totalDamage(bossHits(a));
const bossPowerDps    = (a, b) => powerDamage(bossHits(b)) - powerDamage(bossHits(a));
const bossCondiDps    = (a, b) => condiDamage(bossHits(b)) - condiDamage(bossHits(a));
const dpsSort         = (a, b) => totalDamage(hits(b)) - totalDamage(hits(a));
const powerDpsSort    = (a, b) => powerDamage(hits(b)) - powerDamage(hits(a));
const condiDpsSort    = (a, b) => condiDamage(hits(b)) - condiDamage(hits(a));
const incomingSort    = (a, b) => incomingDamage(b) - incomingDamage(a);
const wastedSort      = (a, b) => wastedTime(b) - wastedTime(a);
const critBossSort    = (a, b) => critRate(bossHits(b)) - critRate(bossHits(a));
const critSort        = (a, b) => critRate(hits(b)) - critRate(hits(a));
const scholarSort     = (a, b) => scholarUptime(hits(b)) - scholarUptime(hits(a));
const scholarBossSort = (a, b) => scholarUptime(bossHits(b)) - scholarUptime(bossHits(a));
const downedSort      = (a, b) => downed(b).length - downed(a).length;

const seconds = time => (time / 1000).toFixed(2);

const professionColour = ({ profession }) => {
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

class SortableComponent extends Component {
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

  th(sort) {
    return ({sortFn, children, ...rest }) => <th {...rest}
      class={[sort, sort.func].indexOf(sortFn) !== -1 ? "selected sortable" : "sortable"}
      onClick={() => this.setSort(sortFn)}>{children}</th>;
  }

  render({ children }, { sort }) {
    if(children.length !== 1) {
       console.error("<SortableComponent /> expects a single child only.");
    }

    return children.map(c => cloneElement(c, { sort, TH: this.th(sort) }))[0];
  }
}

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

class DamageTable extends SortableComponent {
  render({ TH, players, sort }, _, { format: { bossDps, dps, damage } }) {
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
        <TH sortFn={nameSort} title="Character name">Name</TH>
        <TH sortFn={groupSort} title="Subgroup">Group</TH>
        <TH sortFn={bossDpsSort} colspan="3">Boss DPS</TH>
        <TH sortFn={dpsSort} colspan="3">All DPS</TH>
        <TH sortFn={incomingSort} title="Incoming damage">Incoming</TH>
        <TH sortFn={wastedSort} title="Wasted time casting skills which got canceled">Wasted</TH>
        <TH sortFn={critBossSort} colspan="2" title="Percentage of hits which were critical hits">Crits</TH>
        <TH sortFn={scholarBossSort} colspan="2" title="Percentage of hits which potentially benefited from the >90% Scholar rune bonus">Scholar</TH>
        <TH sortFn={downedSort} title="Number of times player got downed."><span class="icon death"></span></TH>
      </tr>
      <tr class="subheading">
        <th colspan="3"></th>
        <TH sortFn={bossDpsSort}>All</TH>
        <TH sortFn={bossPowerDps}>Power</TH>
        <TH sortFn={bossCondiDps}>Condi</TH>
        <TH sortFn={dpsSort}>All</TH>
        <TH sortFn={powerDpsSort}>Power</TH>
        <TH sortFn={condiDpsSort}>Condi</TH>
        <th></th>
        <th></th>
        <TH sortFn={critBossSort}>Boss</TH>
        <TH sortFn={critSort}>All</TH>
        <TH sortFn={scholarBossSort}>Boss</TH>
        <TH sortFn={scholarSort}>All</TH>
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

class BoonTable extends SortableComponent {
  render({ TH, sort, buffs, players }) {
    const sorted  = players.slice().sort(sort);
    const grouped = groupBy(sorted, ({ agent: { subgroup }}) => subgroup);

    const buffsSorted = Object.values(buffs).sort(buffSort);
    const BuffHeading = ({ name, skillId }) => <TH sortFn={() => 0} title={name}>{name}</TH>;

    return <table>
      <tr>
        <th></th>
        <TH sortFn={nameSort} title="Character name">Name</TH>
        <TH sortFn={groupSort} title="Subgroup">Group</TH>
        {buffsSorted.map(BuffHeading)}
      </tr>
      {sorted.map(p => <PlayerBoonRow {...p} buffOrder={buffsSorted} />)}
    </table>;
  }
}

export default class Summary extends Component {
  constructor(props) {
    super(props);

    this.state = {
      section: DAMAGE,
    };
  }

  render({ buffs, encounter, players, enemies }, { section }, { format: { damage } }) {
    let body = null;

    switch(section) {
    case DAMAGE:
      body = <DamageTable players={players} />;
      break;
    case BOONS:
      body = <BoonTable players={players} buffs={buffs} />;
      break;
    default:
      body = <p>Not implemented</p>;
    }

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
        <li class={section === DAMAGE ? "selected" : null} onClick={() => this.setState({ section: DAMAGE })}>Damage</li>
        <li class={section === BOONS ? "selected" : null} onClick={() => this.setState({ section: BOONS })}>Boons</li>
        <li class={section === MECHANICS ? "selected" : null} onClick={() => this.setState({ section: MECHANICS })}>Mechanics</li>
      </ul>
      <SortableComponent>
        {body}
      </SortableComponent>
    </div>;
  }
}