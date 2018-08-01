import { h
       , Component
       } from "preact";

export default class Encounter extends Component {
  render({ boss, success, logName }, _, { encounter: { start }, boss: { end }, format: { time } }) {
    const className = success ? "success" : "failure";
    const startDate = new Date(start);

    return <div class="encounter">
      <h2>{boss} <span class={className}>{success ? "Success" : "Failure"}</span></h2>
      <p>{startDate.toString()}</p>
      <p>Filename: {logName}</p>
      <p class={className}>{success ? "Success" : "Failure"} in {time(end)}</p>
    </div>;
  }
}