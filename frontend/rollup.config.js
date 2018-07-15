import babel      from "rollup-plugin-babel";
import resolve    from "rollup-plugin-node-resolve";
import gzip       from "rollup-plugin-gzip";
import { uglify } from "rollup-plugin-uglify";
import replace    from "rollup-plugin-replace";

const production = process.env.NODE_ENV === "production";

export default {
  input:  "src/index.js",
  output: [
    {
      file:      "dist/index.js",
      sourcemap: true,
      format:    "iife",
      name:      "evtc_rs",
    },
  ],
  plugins: [
    {
      name: "custom-plugin-svg",
      transform: (source, id) => {
        if( ! /\.svg$/.test(id)) {
          return;
        }

        const height = /height="([^"].*?)"/i.exec(source)[1];
        const width  = /width="([^"].*?)"/i.exec(source)[1];

        const viewbox = height && width ? `viewBox="0 0 ${width} ${height}"` : "";

        return {
          code: `import { h } from "preact";
export default function(props) { return ${
  source.replace(/^<\?xml.*?\?>/i, "")
        .replace(/(height|width|xmlns|version|desc)="[^"]*?"/ig, "")
        .replace(/<svg/i, `<svg {...props} preserveAspectRatio="none" ${viewbox}`)}
};`,
          map:  { mappings: "" },
        }
      }
    },
    babel({
      include: [/\.js$/, /\.svg$/],
      babelrc: false,
      presets: [
        ["@babel/preset-env", {
          "modules": false,
          "loose":   true,
          "targets": {
            "node":     "current",
            "browsers": "last 2 versions"
          },
          "exclude": [ "transform-typeof-symbol" ]
        }]
      ],
      plugins: [
        ["@babel/plugin-syntax-jsx"],
        ["@babel/plugin-transform-react-jsx", { "pragma": "h" }],
      ]
    }),
    resolve({
      module:      true,
      jsnext:      true,
      modulesOnly: true,
    }),
  // We only perform the replace in pure production
  ].concat(production ? [
    replace({
      "process.env.NODE_ENV": JSON.stringify("production"),
    }),
  ] : []).concat([
    uglify({
      compress: {
        booleans:      true,
        collapse_vars: true,
        conditionals:  true,
        dead_code:     true,
        evaluate:      true,
        hoist_funs:    true,
        hoist_props:   true,
        hoist_vars:    false,
        if_return:     true,
        inline:        true,
        join_vars:     true,
        keep_fargs:    true,
        keep_fnames:   false,
        loops:         true,
        negate_iife:   true,
        passes:        3,
        properties:    true,
        pure_funcs:    [],
        pure_getters:  true,
        reduce_funcs:  true,
        reduce_vars:   true,
        sequences:     true,
        typeofs:       true,
        unsafe:        true,
        unsafe_proto:  true,
        unused:        true,
        warnings:      true,
      },
      mangle: production ? {
        toplevel:   true,
        reserved:   [],
        properties: {
          regex: /^_/
        },
      } : false,
      output: {
        beautify: !production
      }
    }),
    gzip({
      options: {
        level: 9
      }
    })
  ]),
};
