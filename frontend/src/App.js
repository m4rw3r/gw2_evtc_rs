import { Component
       , h
       } from "preact";
import { HashRouter
       , Link
       , Route } from "react-router-dom";

import PlayerList,
       { TAB_SUMMARY } from "./PlayerList";
import Encounter     from "./Encounter";
import Summary       from "./Summary";
import PlayerSummary from "./PlayerSummary";

// TODO: Use a fragment-router to determine what to show and which settings

export class App extends Component<Data> {
  constructor() {
    super();

    this.state = {
      selected: TAB_SUMMARY,
    };

    this.onSelect = this.onSelect.bind(this);
  }
  getChildContext() {
    const { encounter, enemies } = this.props;

    /*
    let lastBossActive = enemies.reduce((a, { agent: { firstAware } }) => Math.min(a, firstAware), Number.MAX_VALUE);

    const bossDuration = enemies.reduce((a, { agent: { firstAware, lastAware } }) => {
      let r = a + lastAware - firstAware - Math.max(lastBossActive - firstAware, 0);

      lastBossActive = Math.max(lastAware, lastBossActive);

      return r;
    }, 0) / 1000;
    */

    let { start, end } = enemies.reduce(({ start, end }, { agent }) => ({
      start: Math.min(start, agent.firstAware),
      end:   Math.max(end, agent.diedAt || agent.lastAware),
    }), { start: Number.MAX_VALUE, end: 0 });

    const firstHpEvent = ({ series }) => {
      for(let i = 0; i < series.length; i++) {
        if(series[i].health > 0) {
          return series[i].time;
        }
      }

      return Number.MAX_SAFE_INTEGER;
    };
    const duration        = (encounter.logEnd - encounter.logStart) * 1000;
    const firstHpActivity = enemies.reduce((a, b) => Math.min(firstHpEvent(b) * 1000, a), Number.MAX_SAFE_INTEGER) - 1;
    const bossDuration    = (end - firstHpActivity);

    const time = (timestamp, precision=1) => {
      timestamp -= start;
      timestamp /= 1000;

      return `${(timestamp / 60)|0}:${("00" + (timestamp|0) % 60).slice(-2)}${precision ? (timestamp - timestamp|0).toFixed(precision).slice(1) : ""}`;
    };
    const number  = (value, maximumFractionDigits=2) => value.toLocaleString(undefined, { maximumFractionDigits });
    const dps     = total => number(Math.round(total / duration * 1000));
    const bossDps = total => number(Math.round(total / duration * 1000));
    const percent = (fraction, precision=2)  => (fraction * 100).toFixed(precision) + "%";

    return {
      encounter: {
        start:    encounter.logStart * 1000,
        end:      encounter.logEnd * 1000,
        duration: duration,
      },
      boss: {
        firstHpActivity,
        start,
        end,
        duration: bossDuration,
      },
      format: {
        time,
        damage: number,
        number,
        dps,
        bossDps,
        percent,
      },
    };
  }
  onSelect(name) {
    this.setState({
      selected: name,
    });
  }
  render(data, { selected }) {
    const { encounter, players, enemies, skills } = data;
    const player = players.find(p => p.agent.name === selected);

    /*j
    const component = selected === TAB_SUMMARY
      ? <Summary {...data} />
      : <PlayerSummary {...data} key={player.agent.accountName} player={player} />;
    */

    return <HashRouter>
      <div class="evtc">
        <Encounter {...encounter} enemies={enemies} />

        <div class="evtc-body">
          <PlayerList players={players} selected={selected} onSelect={this.onSelect} />

          <section class="evtc-content">
            <Route exact path="/" render={() => <Summary {...data} />} />
            <Route path="/player/:name" render={({ match: { params: { name } } }) => <PlayerSummary {...data} key={name} player={players.find(p => p.agent.name === name)} /> } />
          </section>
        </div>
      </div>
    </HashRouter>
    ;
  }
}