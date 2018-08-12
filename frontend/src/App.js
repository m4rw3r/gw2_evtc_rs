import { Component
       , h
       } from "preact";
import { HashRouter
       , Link
       , Switch
       , Route } from "react-router-dom";

import PlayerList,
       { TAB_SUMMARY } from "./PlayerList";
import Encounter     from "./Encounter";

export class App extends Component<Data> {
  getChildContext() {
    const { buffs, encounter, enemies, players, skills } = this.props;

    // TODO: Maybe perform these calculations in the Rust code?
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

    // TODO: Maybe perform these transforms in the Rust code? (with the exception of the formatting functions)
    return {
      buffs,
      boss: {
        firstHpActivity,
        start,
        end,
        duration: bossDuration,
      },
      encounter: {
        ...encounter,
        start:    encounter.logStart * 1000,
        end:      encounter.logEnd * 1000,
        duration: duration,
      },
      enemies,
      players,
      skills,
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
  render({ encounter, players, enemies, skills, children }) {
    return <div class="evtc">
      <Encounter {...encounter} enemies={enemies} />

      <div class="evtc-body">
        <PlayerList players={players} />
        <section class="evtc-content">
          {children}
        </section>
      </div>
    </div>
    ;
  }
}