import { h
       , Component
       } from "preact";

export default class Summary extends Component {
  render({ encounter, players }, _, { duration, bossDuration }) {
    const playerBossDPS = ({ bossHits }) => (bossHits.power.totalDamage + bossHits.condi.totalDamage) / bossDuration;
    const Player        = (player) => <div>{player.agent.name}: {playerBossDPS(player)}</div>;

    return <div>
      {players.slice().sort((a, b) => playerBossDPS(b) - playerBossDPS(a)).map(Player)}
    </div>;
  }
}