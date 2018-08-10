import { h
       , render
       } from "preact";
import { App } from "./App";

export default function createApp(data: Data, element: Element) {
  console.log(data);

  render(<App {...data} />, element);
}