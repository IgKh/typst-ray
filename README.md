# TypstRay

This is a browser-based tool for visualizing the intermediate document representations of the [Typst](https://typst.app) compiler. Write a piece of Typst source code, and inspect the corresponding AST, content tree and realized paged layout.

You can think of it a little like [Compiler Explorer](https://godbolt.org) or [CppInsights](https://cppinsights.io), but for Typst. Nowhere as fancy though!

The compiler runs entirely in the browser using WebAssembly. There is no backend, and nothing is sent to a server. A public instance is available in https://xray.katvan.app.

## Important Limitations

- Does not support documents using packages or external files.
- Only supports inspecting the paged compilation for now. HTML compilation not yet supported.

## Development

To locally run TypstRay, it requires Node.js, NPM, a Rust toolchain, `wasm-pack` and Python 3.

```sh
npm install
npm run dev          # builds the WASM module (unoptimized) on first run, then starts Vite
```

Other scripts:

```sh
npm run build        # optimized WASM build + production bundle in dist/
npm run check        # type-check
npm run format       # prettier
```

## License

This project is licensed under the GNU Affero General Public License (AGPL), version 3.0 or later. See [LICENSE](LICENSE).
