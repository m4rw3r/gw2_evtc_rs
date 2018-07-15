import { Component
       , h
       , render
       } from "preact";
import PlayerList from "./PlayerList";
import Encounter  from "./Encounter";
import Summary    from "./Summary";

class App extends Component {
  render(data) {
    const { encounter, players, enemies, skills } = data;

    let { start, end } = enemies.reduce(({ start, end }, { agent }) => ({
      start: Math.min(start, agent.firstAware),
      end:   Math.max(end, agent.diedAt || agent.lastAware),
    }), { start: Number.MAX_VALUE, end: 0 });

    const duration     = (encounter.logEnd - encounter.logStart);
    const bossDuration = (end - start) / 1000;

    const totalBossDPS = players.map(player => player.bossHits.power.totalDamage + player.bossHits.condi.totalDamage).reduce((a, b) => a + b, 0) / bossDuration;

    return <div class="evtc">
      <Encounter {...encounter} duration={duration} />

      <div class="evtc-body">
        <PlayerList players={players} totalBossDPS={totalBossDPS} />

        <section class="evtc-content">
          <Summary {...data} duration={duration} bossDuration={bossDuration} />
        </section>
      </div>
    </div>;
  }
}

export default function createApp(data, element) {
  console.log(data);
  render(<App {...data} />, element);
}