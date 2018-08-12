import { h
       , Component
       } from "preact";
import { NavLink } from "react-router-dom";

import { groupBy } from "./util";
import Profession from "./icons/Profession";

export const TAB_SUMMARY = "_SUMMARY";

export default class PlayerList extends Component {
  render({ onSelect, players }, _, { format: { dps } }) {

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

      return <NavLink to={`/players/${name}`} className="player">
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
      </NavLink>;
    };

    const Subgroup = (players) => <div class="subgroup">{players.map(Player)}</div>;

    const grouped = groupBy(players, ({ agent: { subgroup }}) => subgroup).map(group => group.sort((a, b) => a.agent.name.localeCompare(b.agent.name)));

    const totalBossDamage = players.map(player => player.bossHits.power.totalDamage + player.bossHits.condi.totalDamage).reduce((a, b) => a + b, 0);

    return <div class="player-list">
      <NavLink exact to="/" className="player">
        <div class="profession"></div>
        <div class="name">
          <h3>Summary</h3>
          <div class="accountName">
            Boss DPS: {dps(totalBossDamage)}
          </div>
        </div>
        <div class="info"></div>
      </NavLink>

      {grouped.map(Subgroup)}
    </div>;
  }
}
