import { h
       , Component
       } from "preact";

export default class Encounter extends Component {
  render({ boss, success, logName, logStart }, _, { duration }) {
    const className = success ? "success" : "failure";

    const minutes = (duration / 60)|0;
    const seconds = (duration % 60);
    const start   = new Date(logStart * 1000);

    return <div class="encounter">
      <h2 >{boss} <span class={className}>{success ? "Success" : "Failure"}</span></h2>
      <p>Filename: {logName}</p>
      <p>{minutes ? minutes + " minutes " : ""} {seconds.toFixed(1)} seconds</p>
      <p>{start.toString()}</p>
    </div>;
  }
}