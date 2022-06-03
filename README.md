# Speckdrumm
*Audio Spectrum Analyzer for the web, in Rust/WASM*

This is just me tutorialing myself into wasm-bindgen.
(Sorry for the terrible pun, but I couldn't resist.)

Building:
```bash
npm install
npm run build -- --mode production
# Result is in ./dist and could be served with
miniserve dist -p 8080 --index index.html
# Be aware that using the audio API outside of 127.0.0.1 requires https
```

Dev:
```bash
npm run serve -- --mode development
```

Live at https://mimo.liftm.de.
