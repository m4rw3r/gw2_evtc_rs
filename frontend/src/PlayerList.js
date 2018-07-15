import { h
       , Component
       } from "preact";

import Profession from "./icons/Profession";

export const TAB_SUMMARY = "_SUMMARY";

export default class PlayerList extends Component {
  render({ onSelect, players, selected }, _, { bossDuration }) {

    const Player = ({ agent }) => {
      const { name
            , accountName
            , subgroup
            , profession
            , conditionDmg
            , toughness
            , concentration
            , healing
            , isPov
            , diedAt
            } = agent;

      return <div class={`player${selected === name ? " active" : ""}`} onClick={() => onSelect(name)}>
        <Profession class="profession" profession={profession} />

        <div class="name">
          <h3>{name}</h3>

          <div class="accountName">{accountName}</div>
        </div>

        <div class="info">
          {conditionDmg ? <span class="icon condiDmg" title={`Condition Damage: ${conditionDmg}`}></span> : null}
          {concentration ? <span class="icon concentration" title={`Concentration: ${concentration}`}></span> : null}
          {healing ? <span class="icon healing" title={`Healing Power: ${healing}`}></span> : null}
          {toughness ? <span class="icon toughness" title={`Toughness: ${toughness}`}></span> : null}
          {isPov ? <span class="icon revealed" title="Point of View"></span> : null}
          {diedAt ? <span class="icon death" title="Died"></span> : null}
        </div>
      </div>;
    };

    const Subgroup = (players) => <div class="subgroup">{players.map(Player)}</div>;

    const grouped = players.slice()
           .sort((a, b) => a.agent.name.localeCompare(b.agent.name))
           .reduce((g, p) => {
             const { agent: { subgroup } } = p;
      g[subgroup] = g[subgroup] || [];
      g[subgroup].push(p);

      return g;
    }, {});

    const subgroups    = Object.keys(grouped).sort().map(key => grouped[key]);
    const totalBossDPS = players.map(player => player.bossHits.power.totalDamage + player.bossHits.condi.totalDamage).reduce((a, b) => a + b, 0) / bossDuration;

    return <div class="player-list">
      <div class={`player${selected === TAB_SUMMARY ? " active" : ""}`} onClick={() => onSelect(TAB_SUMMARY)}>
        <div class="profession"></div>
        <div class="name">
          <h3>Summary</h3>
          <div class="accountName">
            Boss DPS: {totalBossDPS.toFixed(0)}
          </div>
        </div>
        <div class="info"></div>
      </div>

      {subgroups.map(Subgroup)}
    </div>;
  }
}
