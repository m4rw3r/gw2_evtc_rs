import { h
       , cloneElement
       , Component
       } from "preact";

import { min, max } from "d3-array";
import { scaleLinear }   from "d3-scale";
import { line }     from "d3-shape";

const PADDING_DEFAULT = 20;

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

export const GroupedGraphs = ({ mouseX, children, padding, width, height, start, end, x: _, xScale, yScale, ...rest }) => {
  const allSeries = children.map(c => c.attributes && c.attributes.series ? c.attributes.series : []);
  const x         = n => xScale(fst(n));

  yScale.domain([max(allSeries.map(s => max(s, snd))), 0]);

  return <g>
    {children.map(c => cloneElement(c, { start, padding, width, height, start, end, x, xScale, yScale: yScale.copy() }))}
  </g>;
};

export class Axis extends Component {
  render({ yScale, padding, format=(x => x), xScale: _x, width: _w, height: _h, start: _s, end: _e, ...rest }) {
    const range  = yScale.ticks();
    const domain = yScale.domain();
    const half   = padding / 4;

    return <g {...rest} fill="none">
      <path stroke="#000" d={`M${padding},${yScale(domain[0])}V${yScale(domain[1])}`} />

      {range.map(v => <g transform={`translate(${padding},${yScale(v)})`}>
        <line stroke="#000" x2={-half} />
        <text fill="#000" x={half} y={half}>{format(v)}</text>
      </g>)}
    </g>;
  }
}

export class TimeAxis extends Component {
  render({ xScale, height, padding, mouseX, yScale: _y, width: _w, start: _s, end: _e, ...rest }, _, { format: { time } }) {
    const range  = xScale.ticks();
    const domain = xScale.domain();
    const half   = padding / 2;

    // TODO: Configurable
    return <g {...rest} fill="none" transform={`translate(0,${height - padding})`}>
      <path stroke="#000" d={`M${xScale(domain[0])},0H${xScale(domain[1])}`} />

      {range.map(t => <g transform={`translate(${xScale(t)},0)`}>
        <line stroke="#000" y2={half} />
        <text fill="#000" x={half / 2} y={half} dy="0.25em">{time(t * 1000, 0)}</text>
      </g>)}
    </g>;
  }
}

export class ResponsiveGraph extends Component {
  constructor(props) {
    super(props);

    this.state = {
      width: 0,
    };

    this.resize = this.resize.bind(this);
  }
 
  componentDidMount() {
    this.resize()

    window.addEventListener("resize", this.resize);
  }

  componentWillUnmount() {
    window.removeEventListener("resize", this.resize);
  }

  resize() {
    if( ! this.el) {
      return;
    }

    const { width } = this.state;
    const newWidth  = this.el.getBoundingClientRect().width;

    if(newWidth != width) {
      this.setState({
        width: newWidth,
      });
    }
  }

  render({ children, ...rest }) {
    const className = (rest.class || rest.className || "") + " responsive-graph";

    return <div {...rest} class={className} ref={el => this.el = el}>
      {children.map(c => cloneElement(c, { width: c.attributes.width ? Math.min(c.attributes.width, this.state.width) : c.attributes.width }))}
    </div>;
  }
}

export class Graph extends Component {
  constructor(props) {
    super(props);

    this.state = {
      mouseX: null,
    };
    this.mouseleave = this.mouseleave.bind(this);
    this.mousemove  = this.mousemove.bind(this);
  }
  mousemove(e) {
    const { left, top } = this.base.getBoundingClientRect();
    const padding = this.props.padding || PADDING_DEFAULT;

    const coords = {
      x: Math.min(this.props.width - padding, Math.max(e.clientX - left - padding, 0)),
      y: Math.min(this.props.height - padding, Math.max(e.clientY - top + padding, 0))
    };

    this.setState({
      mouseX: x,
    });
  }
  mouseleave() {
    this.setState({
      mouseX: null,
    });
  }
  render({ start, end, width, height, padding=PADDING_DEFAULT, children, ...rest }, { mouseX }) {
    // TODO: Filter based on start/end values of time
    const xScale = scaleLinear().domain([start|0, Math.ceil(end)]).range([padding, width - padding]);
    const x      = n => xScale(time(n));
    const yScale = scaleLinear().range([padding, height - padding]);

    return <svg {...rest} width={width} height={height} onMouseMove={this.mousemove} onMouseLeave={this.mouseleave}>
      {children.map(c => cloneElement(c, { mouseX, padding, width, height, start, end, x, xScale, yScale: yScale.copy() }))}
    </svg>;
  }
}
