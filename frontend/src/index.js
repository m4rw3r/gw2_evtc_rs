import { Component
       , h
       , render
       } from "preact";

import PlayerList,
       { TAB_SUMMARY } from "./PlayerList";
import Encounter     from "./Encounter";
import Summary       from "./Summary";
import PlayerSummary from "./PlayerSummary";

class App extends Component {
  constructor() {
    super();

    this.state = {
      selected: TAB_SUMMARY,
    };

    this.onSelect = this.onSelect.bind(this);
  }
  getChildContext() {
    const { encounter, enemies } = this.props;

    let { start, end } = enemies.reduce(({ start, end }, { agent }) => ({
      start: Math.min(start, agent.firstAware),
      end:   Math.max(end, agent.diedAt || agent.lastAware),
    }), { start: Number.MAX_VALUE, end: 0 });

    const duration     = (encounter.logEnd - encounter.logStart);
    const bossDuration = (end - start) / 1000;

    return {
      duration,
      bossDuration,
    };
  }
  onSelect(name) {
    this.setState({
      selected: name,
    });
  }
  render(data, { selected }) {
    const { encounter, players, enemies, skills } = data;

    const component = selected === TAB_SUMMARY
      ? <Summary {...data} />
      : <PlayerSummary {...data} player={players.find(p => p.agent.name === selected)} />;

    return <div class="evtc">
      <Encounter {...encounter} />

      <div class="evtc-body">
        <PlayerList players={players} selected={selected} onSelect={this.onSelect} />

        <section class="evtc-content">
          {component}
        </section>
      </div>
    </div>;
  }
}

export default function createApp(data, element) {
  console.log(data);
  render(<App {...data} />, element);
}