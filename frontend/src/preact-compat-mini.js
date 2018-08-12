import { h
       , createElement
       , cloneElement
       , Component
       , render
       , rerender
       , options
       } from "preact";

const oldVnode = options.vnode;

options.vnode = vnode => {
  const children = vnode.children;

  if(children && children.length === 1 && typeof children[0] === "function") {
    vnode.children = children[0];
  }

  vnode.props = vnode.attributes;

  oldVnode && oldVnode(vnode);
};

const isValidElement = el => typeof el === "object" && el.nodeName;

const Children = {
  only:    c => Array.isArray(c) ? c[0] : c,
  count:   c => c.length,
  forEach: (c, fn) => c.forEach(fn),
};

export default {
  h,
  createElement,
  cloneElement,
  Component,
  Children,
  isValidElement,
  render,
  rerender,
  options,
};

export{
  h,
  createElement,
  cloneElement,
  Component,
  Children,
  isValidElement,
  render,
  rerender,
  options,
};