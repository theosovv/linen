const { readFileSync } = require("fs");
const resolve = require('@rollup/plugin-node-resolve');
const commonjs = require('@rollup/plugin-commonjs');
const typescript = require('@rollup/plugin-typescript');
const { dts } = require('rollup-plugin-dts');

const pkg = JSON.parse(readFileSync('./package.json', 'utf-8'));

const external = [
  ...Object.keys(pkg.dependencies || {}),
  ...Object.keys(pkg.peerDependencies || {}),
];

const plugins = [
  resolve({ preferBuiltins: true }),
  commonjs(),
  typescript({
    tsconfig: './tsconfig.build.json',
    exclude: ['**/*.test.ts', '**/*.spec.ts']
  })
];

module.exports = [
  {
    input: 'src/index.ts',
    output: {
      file: 'dist/index.esm.js',
      format: 'esm',
      sourcemap: true
    },
    external,
    plugins
  },
  {
    input: 'src/index.ts',
    output: {
      file: 'dist/index.cjs.js',
      format: 'cjs',
      sourcemap: true
    },
    external,
    plugins
  },
  {
    input: 'src/index.ts',
    output: {
      file: 'dist/index.umd.js',
      format: 'umd',
      name: 'Linen',
      sourcemap: true
    },
    plugins
  },
  {
    input: 'src/index.ts',
    output: {
      file: 'dist/index.d.ts',
      format: 'esm'
    },
    plugins: [dts()]
  },
  {
    input: 'src/jsx-runtime.ts',
    output: {
      file: 'dist/jsx-runtime.js',
      format: 'esm',
      sourcemap: true
    },
    external,
    plugins
  },
  {
    input: 'src/jsx-runtime.ts',
    output: {
      file: 'dist/jsx-runtime.esm.js',
      format: 'esm',
      sourcemap: true
    },
    external,
    plugins
  },
  {
    input: 'src/jsx-runtime.ts',
    output: {
      file: 'dist/jsx-runtime.cjs.js',
      format: 'cjs',
      sourcemap: true
    },
    external,
    plugins
  },
  {
    input: 'src/jsx-runtime.ts',
    output: {
      file: 'dist/jsx-runtime.umd.js',
      format: 'umd',
      name: 'Linen',
      sourcemap: true
    },
    plugins
  },
  {
    input: 'src/jsx-runtime.ts',
    output: {
      file: 'dist/jsx-runtime.d.ts',
      format: 'esm'
    },
    plugins: [dts()]
  }
];