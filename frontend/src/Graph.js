import { h
       , Component
       } from "preact";

import { min, max } from "d3-array";
import { scaleLinear }   from "d3-scale";
import { line }     from "d3-shape";

const time   = ({ time })   => time;
export const health = ({ health }) => health;
export const damage = ({ damage }) => damage;
export const downed = ({ downed }) => downed;

const renderLine = ({ series, time, yScale, prop }) => {
    yScale = yScale.copy().domain([max(series, prop), 0]);

    const lineGraph = line().x(time).y(n => yScale(prop(n)));

    return <path class="line" d={lineGraph(series.filter(prop))} />
}

export default class Graph extends Component {
  render(props) {
    let { series, width, height, start, end, padding=10, ...rest } = props;

    start = Math.max(start || 0, min(series, time));
    end   = Math.min(end || Number.MAX_SAFE_INTEGER, max(series, time));

    const xScale = scaleLinear().domain([start, end]).range([padding, width - padding]);
    const x      = n => xScale(time(n));
    const yScale = scaleLinear().range([padding, height - padding]);

    return <svg {...rest} width={width} height={height}>
      <g>
        {renderLine({ series, time: x, yScale, prop: health })}
        {renderLine({ series, time: x, yScale, prop: damage })}
      </g>
    </svg>;
  }
}
