import { h
       , cloneElement
       , Component
       } from "preact";

import { min, max } from "d3-array";
import { scaleLinear }   from "d3-scale";
import { line }     from "d3-shape";

const fst  = ([x, _]) => x;
const snd  = ([_, x]) => x;
const time = ({ time })   => time;

export const health        = ({ health }) => health / 100;
export const damageSeries  = series => series.map(({ time, damage }) => [time, damage]);
export const bossDmgSeries = series => series.map(({ time, bossDmg }) => [time, bossDmg]);
export const fulltimeAvg   = (series: [number, number], start) => {
  let acc  = 0;

  let r = series.map(([time, v]) => {
    if(time < start) {
      return 0;
    }

    acc += v;

    return [time, acc / Math.max(time - start, 1)];
  }).filter(x => x);

  return r;
};
export const downed       = ({ downed }) => downed;

const renderLine = ({ series, time, yScale, prop }) => {
    yScale = yScale.copy().domain([max(series, prop), 0]);

    const lineGraph = line().x(time).y(n => yScale(prop(n)));

    return <path class="line" d={lineGraph(series.filter(prop))} />
}

// TODO: Generalize these
export const HealthGraph = ({ series, start, end, x, xScale, yScale, ...rest }) => {
  yScale.domain([100, 0]);

  const lineGraph = line().x(x).y(n => yScale(health(n)));

  return <path d={lineGraph(series.filter(health))} {...rest} />
};

export const DPSGraph = ({ series, x, start, end, xScale, yScale, ...rest }) => {
  const lineGraph = line().x(x).y(([_, n]) => yScale(n));

  return <path d={lineGraph(series)} {...rest} />
}

export const GroupedGraphs = ({ children, start, end, x: _, xScale, yScale, ...rest }) => {
  const allSeries = children.map(c => c.attributes.series);
  const x         = n => xScale(fst(n));

  yScale.domain([max(allSeries.map(s => max(s, snd))), 0]);

  return <g>
    {children.map(c => cloneElement(c, { start, end, x, xScale, yScale: yScale.copy() }))}
  </g>;
};

export class Graph extends Component {
  render({ start, end, width, height, padding=10, children, ...rest }) {
    // TODO: Filter based on start/end values of time
    const xScale = scaleLinear().domain([start|0, Math.ceil(end)]).range([padding, width - padding]);
    const x      = n => xScale(time(n));
    const yScale = scaleLinear().range([padding, height - padding]);

    return <svg {...rest} width={width} height={height}>
      {children.map(c => cloneElement(c, { start, end, x, xScale, yScale: yScale.copy() }))}
    </svg>;
  }
}
