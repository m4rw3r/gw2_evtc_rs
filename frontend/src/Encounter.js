import { h
       , Component
       } from "preact";

export default class Encounter extends Component {
  render({ boss, success, logName, enemies }, _, { encounter: { start }, boss: { end }, format: { time, percent } }) {
    const className = success ? "success" : "failure";
    const startDate = new Date(start);
    const lowestHp  = [].concat.apply([], enemies.map(e => e.series.map(s => s.health).filter(h => h > 0))).reduce((a, h) => Math.min(a, h), Number.MAX_SAFE_INTEGER);

    return <div class="encounter">
      <h2>{boss} <span class={className}>{success ? "Success" : "Failure"}</span></h2>
      <p>{startDate.toString()}</p>
      <p>Filename: {logName}</p>
      <p class={className}>{success ? "Success" : "Failure"} in {time(end)}</p>
      {success ? null : <p>Lowest HP: {percent(lowestHp / 10000)}</p>}
    </div>;
  }
}