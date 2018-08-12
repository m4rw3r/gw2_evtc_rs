import { h
       , render
       , Component
       } from "preact";
import { App } from "./App";
import { Summary
       , BoonTable
       , DamageTable
       } from "./Summary";
import { PlayerSummary
       , AgentSummary
       , ActivationLog
       } from "./PlayerSummary";
import { HashRouter
       , Link
       , Switch
       , Route as ReactRouterRoute } from "react-router-dom";

/**
 * Repacement for react-router's Route, provides a more declarative approach by being able to
 * pass-through props.
 */
class Route extends Component {
  render({ computedMatch, path, exact, strict, sensitive, component, render, children, location, ...rest }) {
    return h(ReactRouterRoute, {
      computedMatch,
      path,
      exact,
      strict,
      sensitive,
      render: props => {
        if(component) {
          return h(component, Object.assign(rest, props), children);
        }

        if(render) {
          return render(Object.assign(rest, props));
        }

        if(typeof children === "function") {
          return children(Object.assign(rest, props));
        }

        return null;
      }
    })
  }
}

const app = (data: Data) => <HashRouter>
  <App {...data}>
    <Switch>
      <Route path="/players/:name" component={PlayerSummary}>
        <Route exact path="/players/:name" component={AgentSummary} />
        <Route path="/players/:name/agents/:speciesId" component={AgentSummary} />
        <ActivationLog />
      </Route>

      <Route path="/" component={Summary}>
        <Route path="/boons" component={BoonTable} />
        <Route exact path="/" component={DamageTable} />
      </Route>
    </Switch>
  </App>
</HashRouter>;

export default function createApp(data: Data, element: Element) {
  console.log(data);

  render(app(data), element);
}