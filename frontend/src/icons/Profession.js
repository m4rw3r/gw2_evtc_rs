import { Component
       , h
       } from "preact";

import Berserker from "./professions/Berserker.svg"
import Chronomancer from "./professions/Chronomancer.svg"
import Daredevil from "./professions/Daredevil.svg"
import Deadeye from "./professions/Deadeye.svg"
import Dragonhunter from "./professions/Dragonhunter.svg"
import Druid from "./professions/Druid.svg"
import Elementalist from "./professions/Elementalist.svg"
import Engineer from "./professions/Engineer.svg"
import Firebrand from "./professions/Firebrand.svg"
import Guardian from "./professions/Guardian.svg"
import Herald from "./professions/Herald.svg"
import Holosmith from "./professions/Holosmith.svg"
import Mesmer from "./professions/Mesmer.svg"
import Mirage from "./professions/Mirage.svg"
import Necromancer from "./professions/Necromancer.svg"
import Ranger from "./professions/Ranger.svg"
import Reaper from "./professions/Reaper.svg"
import Renegade from "./professions/Renegade.svg"
import Revenant from "./professions/Revenant.svg"
import Scourge from "./professions/Scourge.svg"
import Scrapper from "./professions/Scrapper.svg"
import Soulbeast from "./professions/Soulbeast.svg"
import Spellbreaker from "./professions/Spellbreaker.svg"
import Tempest from "./professions/Tempest.svg"
import Thief from "./professions/Thief.svg"
import Warrior from "./professions/Warrior.svg"
import Weaver from "./professions/Weaver.svg"

const professions = {
  Berserker,
  Chronomancer,
  Daredevil,
  Deadeye,
  Dragonhunter,
  Druid,
  Elementalist,
  Engineer,
  Firebrand,
  Guardian,
  Herald,
  Holosmith,
  Mesmer,
  Mirage,
  Necromancer,
  Ranger,
  Reaper,
  Renegade,
  Revenant,
  Scourge,
  Scrapper,
  Soulbeast,
  Spellbreaker,
  Tempest,
  Thief,
  Warrior,
  Weaver,
};

export default class Profession extends Component {
  render(props) {
    const { profession, ...rest } = props;

    const P = professions[profession];

    return P ? <P {...rest} /> : <div {...rest}></div>;
  }
}