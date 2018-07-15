import { h
       , Component
       } from "preact";

import Profession from "./icons/Profession";

export default class PlayerList extends Component {
  render({ onSelect, players, totalBossDPS }) {

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

            console.log(agent);

      return <div class="player">
        <Profession class="profession" profession={profession} />

        <div class="name">
          <h3>{name}</h3>

          <div class="accountName">{accountName}</div>
        </div>

        <div class="info">
          {conditionDmg ? <span class="icon condiDmg"></span> : null}
          {concentration ? <span class="icon concentration"></span> : null}
          {healing ? <span class="icon healing"></span> : null}
          {toughness ? <span class="icon toughness"></span> : null}
          {isPov ? <span class="icon revealed"></span> : null}
          {diedAt ? <span class="icon death"></span> : null}
        </div>
      </div>;
    };

    // TODO: Group by subgroup
    return <div class="player-list">
      <div class="player">
        <div class="profession"></div>
        <div class="name">
          <h3>Summary</h3>
          <div class="accountName">
            Boss DPS: {totalBossDPS.toFixed(0)}
          </div>
        </div>
        <div class="info"></div>
      </div>

      {players.map(Player)}
    </div>;
  }
}
